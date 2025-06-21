use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;
use anyhow::{Result, anyhow};
use crate::models::AudioDevice;

// Simple audio engine that doesn't store streams in shared state
pub struct AudioEngine {
    host: Host,
    recording_state: Arc<Mutex<RecordingState>>,
}

struct RecordingState {
    is_recording: bool,
    start_time: Option<Instant>,
    writer_thread: Option<thread::JoinHandle<()>>,
    recording_file_path: Option<String>,
    // Store a stop signal instead of the actual streams
    stop_sender: Option<Sender<()>>,
}

#[derive(Clone)]
struct AudioSample {
    data: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        
        let recording_state = Arc::new(Mutex::new(RecordingState {
            is_recording: false,
            start_time: None,
            writer_thread: None,
            recording_file_path: None,
            stop_sender: None,
        }));

        Ok(AudioEngine {
            host,
            recording_state,
        })
    }

    pub fn get_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut devices = Vec::new();

        // Get input devices (microphones)
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let is_default = self.host.default_input_device()
                        .map(|d| d.name().unwrap_or_default() == name)
                        .unwrap_or(false);
                    
                    devices.push(AudioDevice {
                        name,
                        is_default,
                        device_type: "input".to_string(),
                    });
                }
            }
        }

        // Get output devices (for loopback recording)
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    let is_default = self.host.default_output_device()
                        .map(|d| d.name().unwrap_or_default() == name)
                        .unwrap_or(false);
                    
                    devices.push(AudioDevice {
                        name,
                        is_default,
                        device_type: "output".to_string(),
                    });
                }
            }
        }

        Ok(devices)
    }    pub fn start_recording(&self, file_path: &str) -> Result<()> {
        let mut state = self.recording_state.lock().unwrap();
        
        if state.is_recording {
            return Err(anyhow!("Already recording"));
        }

        // Create audio channel for communication between streams and writer
        let (audio_sender, receiver) = mpsc::channel::<AudioSample>();
        let (stop_sender, stop_receiver) = mpsc::channel::<()>();

        // Start the audio writer thread
        let file_path = file_path.to_string();
        let writer_thread = thread::spawn(move || {
            Self::audio_writer_thread(receiver, &file_path);
        });

        // Create a new host for the audio thread instead of cloning
        let audio_thread = thread::spawn(move || {
            let host = cpal::default_host();
            Self::audio_recording_thread(host, audio_sender, stop_receiver);
        });

        state.is_recording = true;
        state.start_time = Some(Instant::now());
        state.writer_thread = Some(writer_thread);
        state.recording_file_path = Some(file_path.to_string());
        state.stop_sender = Some(stop_sender);

        // We need to keep the audio thread alive, but we can't store it in state
        // For now, we'll detach it - in a production app you'd want better lifecycle management
        std::mem::forget(audio_thread);

        Ok(())
    }

    pub fn stop_recording(&self) -> Result<i32> {
        let mut state = self.recording_state.lock().unwrap();
        
        if !state.is_recording {
            return Err(anyhow!("Not currently recording"));
        }

        // Send stop signal to audio thread
        if let Some(stop_sender) = state.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        let duration = if let Some(start_time) = state.start_time {
            start_time.elapsed().as_secs() as i32
        } else {
            0
        };

        // Wait for writer thread to finish
        if let Some(writer_thread) = state.writer_thread.take() {
            let _ = writer_thread.join();
        }

        state.is_recording = false;
        state.start_time = None;
        state.recording_file_path = None;

        Ok(duration)
    }

    fn audio_recording_thread(
        host: Host,
        audio_sender: Sender<AudioSample>,
        stop_receiver: Receiver<()>,
    ) {
        // Get default input device (microphone)
        let input_device = match host.default_input_device() {
            Some(device) => device,
            None => {
                eprintln!("No default input device available");
                return;
            }
        };

        // Create input stream
        let stream = match Self::create_input_stream_static(&input_device, audio_sender) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to create input stream: {}", e);
                return;
            }
        };

        // Start the stream
        if let Err(e) = stream.play() {
            eprintln!("Failed to start audio stream: {}", e);
            return;
        }

        // Keep the stream alive until stop signal is received
        let _ = stop_receiver.recv();
        
        // Stream is automatically stopped when dropped
        drop(stream);
    }

    fn create_input_stream_static(
        device: &Device,
        sender: Sender<AudioSample>,
    ) -> Result<cpal::Stream> {
        let config = device.default_input_config()?;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::create_input_stream_typed_static::<f32>(device, &config.into(), sender)?,
            cpal::SampleFormat::I16 => Self::create_input_stream_typed_static::<i16>(device, &config.into(), sender)?,
            cpal::SampleFormat::U16 => Self::create_input_stream_typed_static::<u16>(device, &config.into(), sender)?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        Ok(stream)
    }

    fn create_input_stream_typed_static<T>(
        device: &Device,
        config: &cpal::StreamConfig,
        sender: Sender<AudioSample>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample + Send + 'static,
        f32: cpal::FromSample<T>,
    {
        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let samples: Vec<f32> = data.iter().map(|&sample| cpal::Sample::from_sample(sample)).collect();
                
                let audio_sample = AudioSample {
                    data: samples,
                    sample_rate,
                    channels,
                };

                // Send audio data to writer thread
                if sender.send(audio_sample).is_err() {
                    // Writer thread has stopped, stream should stop too
                }
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    fn audio_writer_thread(receiver: Receiver<AudioSample>, file_path: &str) {
        // Initialize with default values, will be updated with first sample
        let mut writer: Option<WavWriter<std::io::BufWriter<std::fs::File>>> = None;
        let mut sample_count = 0u32;

        while let Ok(audio_sample) = receiver.recv() {
            // Initialize writer with first sample's parameters
            if writer.is_none() {
                let spec = WavSpec {
                    channels: audio_sample.channels,
                    sample_rate: audio_sample.sample_rate,
                    bits_per_sample: 32, // f32 samples
                    sample_format: hound::SampleFormat::Float,
                };

                match WavWriter::create(file_path, spec) {
                    Ok(w) => writer = Some(w),
                    Err(e) => {
                        eprintln!("Failed to create WAV writer: {}", e);
                        break;
                    }
                }
            }

            // Write samples
            if let Some(ref mut w) = writer {
                for sample in audio_sample.data {
                    if let Err(e) = w.write_sample(sample) {
                        eprintln!("Failed to write audio sample: {}", e);
                        break;
                    }
                    sample_count += 1;
                }
            }
        }

        // Finalize the WAV file
        if let Some(writer) = writer {
            if let Err(e) = writer.finalize() {
                eprintln!("Failed to finalize WAV file: {}", e);
            } else {
                println!("Audio recording saved to {} ({} samples)", file_path, sample_count);
            }
        }
    }
}
