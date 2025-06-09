use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, SupportedStreamConfig};
use hound::{WavSpec, WavWriter};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use crate::models::AudioDevice;

pub struct AudioEngine {
    host: Host,
    recording_state: Arc<Mutex<RecordingState>>,
}

struct RecordingState {
    is_recording: bool,
    start_time: Option<Instant>,
    _mic_stream: Option<Stream>,
    _system_stream: Option<Stream>,
    writer_thread: Option<thread::JoinHandle<()>>,
    audio_sender: Option<Sender<AudioSample>>,
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
            _mic_stream: None,
            _system_stream: None,
            writer_thread: None,
            audio_sender: None,
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
    }

    pub fn start_recording(&mut self, file_path: &str) -> Result<()> {
        let mut state = self.recording_state.lock().unwrap();
        
        if state.is_recording {
            return Err(anyhow!("Already recording"));
        }

        // Create audio channel for communication between streams and writer
        let (sender, receiver) = mpsc::channel::<AudioSample>();

        // Start the audio writer thread
        let file_path = file_path.to_string();
        let writer_thread = thread::spawn(move || {
            Self::audio_writer_thread(receiver, &file_path);
        });

        // Get default input device (microphone)
        let input_device = self.host.default_input_device()
            .ok_or_else(|| anyhow!("No default input device available"))?;

        // Get default output device (for loopback)
        let output_device = self.host.default_output_device()
            .ok_or_else(|| anyhow!("No default output device available"))?;

        // Start microphone stream
        let mic_stream = self.create_input_stream(&input_device, sender.clone())?;
        
        // Start system audio loopback stream (this is platform-specific and complex)
        // For now, we'll focus on microphone recording
        // In a full implementation, you'd need platform-specific code for system audio capture
        let system_stream = None; // TODO: Implement system audio loopback

        mic_stream.play()?;

        state.is_recording = true;
        state.start_time = Some(Instant::now());
        state._mic_stream = Some(mic_stream);
        state._system_stream = system_stream;
        state.writer_thread = Some(writer_thread);
        state.audio_sender = Some(sender);

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<i32> {
        let mut state = self.recording_state.lock().unwrap();
        
        if !state.is_recording {
            return Err(anyhow!("Not currently recording"));
        }

        let duration = state.start_time
            .map(|start| start.elapsed().as_secs() as i32)
            .unwrap_or(0);

        // Stop streams
        state._mic_stream = None;
        state._system_stream = None;

        // Close audio channel
        state.audio_sender = None;

        // Wait for writer thread to finish
        if let Some(thread) = state.writer_thread.take() {
            let _ = thread.join();
        }

        state.is_recording = false;
        state.start_time = None;

        Ok(duration)
    }

    fn create_input_stream(&self, device: &Device, sender: Sender<AudioSample>) -> Result<Stream> {
        let config = device.default_input_config()?;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.create_input_stream_typed::<f32>(device, &config.into(), sender)?,
            cpal::SampleFormat::I16 => self.create_input_stream_typed::<i16>(device, &config.into(), sender)?,
            cpal::SampleFormat::U16 => self.create_input_stream_typed::<u16>(device, &config.into(), sender)?,
            _ => return Err(anyhow!("Unsupported sample format")),
        };

        Ok(stream)
    }

    fn create_input_stream_typed<T>(
        &self,
        device: &Device,
        config: &cpal::StreamConfig,
        sender: Sender<AudioSample>,
    ) -> Result<Stream>
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

                let _ = sender.send(audio_sample);
            },
            |err| eprintln!("Audio input error: {}", err),
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

