//! Stand‑alone audio capture using CPAL + Hound.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream};
use hound::{WavSpec, WavWriter};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use crate::models::AudioDevice;

/* ---------------------------- shared state --------------------------- */

pub struct AudioEngine {
    host: Host,
    state: Arc<Mutex<RecordingState>>,
}

struct RecordingState {
    is_recording: bool,
    start_time: Option<Instant>,
    writer_thread: Option<thread::JoinHandle<()>>,
    stop_tx: Option<Sender<()>>,
    recording_file_path: Option<String>,
}

#[derive(Clone)]
struct AudioSample {
    data: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

/* -------------------------------------------------------------------- */

impl AudioEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            host: cpal::default_host(),
            state: Arc::new(Mutex::new(RecordingState {
                is_recording: false,
                start_time: None,
                writer_thread: None,
                stop_tx: None,
                recording_file_path: None,
            })),
        })
    }

    /* ----------------------------- devices ---------------------------- */

    pub fn get_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        let mut out = Vec::new();

        let default_in = self
            .host
            .default_input_device()
            .and_then(|d| d.name().ok());

        if let Ok(devs) = self.host.input_devices() {
            for d in devs {
                if let Ok(name) = d.name() {
                    out.push(AudioDevice {
                        name,
                        is_default: default_in.as_deref() == Some(&name),
                        device_type: "input".into(),
                    });
                }
            }
        }

        let default_out = self
            .host
            .default_output_device()
            .and_then(|d| d.name().ok());

        if let Ok(devs) = self.host.output_devices() {
            for d in devs {
                if let Ok(name) = d.name() {
                    out.push(AudioDevice {
                        name,
                        is_default: default_out.as_deref() == Some(&name),
                        device_type: "output".into(),
                    });
                }
            }
        }

        Ok(out)
    }

    /* --------------------------- record / stop ------------------------ */

    pub fn start_recording(&self, file_path: &str) -> Result<()> {
        let mut st = self.state.lock().map_err(|_| anyhow!("poisoned"))?;
        if st.is_recording {
            return Err(anyhow!("already recording"));
        }

        // Channels
        let (data_tx, data_rx) = mpsc::channel::<AudioSample>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // Writer thread
        let writer_path = file_path.to_string();
        let writer = thread::spawn(move || Self::writer_thread(data_rx, writer_path));

        // Capture thread
        let capture_host = cpal::default_host();
        thread::spawn(move || Self::capture_thread(capture_host, data_tx, stop_rx));
        /* detached */

        // Store state
        st.is_recording = true;
        st.start_time = Some(Instant::now());
        st.writer_thread = Some(writer);
        st.stop_tx = Some(stop_tx);
        st.recording_file_path = Some(file_path.to_string());

        Ok(())
    }

    pub fn stop_recording(&self) -> Result<i32> {
        let mut st = self.state.lock().map_err(|_| anyhow!("poisoned"))?;
        if !st.is_recording {
            return Err(anyhow!("not recording"));
        }

        if let Some(tx) = st.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(w) = st.writer_thread.take() {
            let _ = w.join();
        }

        let secs = st
            .start_time
            .map(|t| t.elapsed().as_secs() as i32)
            .unwrap_or(0);

        st.is_recording = false;
        st.start_time = None;
        st.recording_file_path = None;

        Ok(secs)
    }

    /* ------------------------ internal helpers ------------------------ */

    fn capture_thread(host: Host, tx: Sender<AudioSample>, stop_rx: Receiver<()>) {
        let device = match host.default_input_device() {
            Some(d) => d,
            None => {
                eprintln!("no default input device");
                return;
            }
        };

        let stream = match Self::build_stream(&device, tx) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("stream error: {e}");
                return;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("could not play stream: {e}");
            return;
        }

        // Block until stop signal
        let _ = stop_rx.recv();
        /* stream drops here */
    }

    fn build_stream(device: &Device, tx: Sender<AudioSample>) -> Result<Stream> {
        let cfg = device.default_input_config()?;
        let sample_rate = cfg.sample_rate().0;
        let channels = cfg.channels();

        fn make<T>(
            dev: &Device,
            cfg: &cpal::StreamConfig,
            tx: Sender<AudioSample>,
            sr: u32,
            ch: u16,
        ) -> Result<Stream>
        where
            T: cpal::Sample + cpal::SizedSample + Send + 'static,
            f32: cpal::FromSample<T>,
        {
            let stream = dev
                .build_input_stream(
                    cfg,
                    move |data: &[T], _| {
                        let v: Vec<f32> =
                            data.iter().map(|s| cpal::Sample::from_sample(*s)).collect();
                        let _ = tx.send(AudioSample {
                            data: v,
                            sample_rate: sr,
                            channels: ch,
                        });
                    },
                    |e| eprintln!("stream callback error: {e}"),
                    None,
                )
                .map_err(|e| anyhow!(e))?;
            Ok(stream)
        }

        let stream = match cfg.sample_format() {
            cpal::SampleFormat::F32 => make::<f32>(&device, &cfg.clone().into(), tx, sample_rate, channels)?,
            cpal::SampleFormat::I16 => make::<i16>(&device, &cfg.clone().into(), tx, sample_rate, channels)?,
            cpal::SampleFormat::U16 => make::<u16>(&device, &cfg.clone().into(), tx, sample_rate, channels)?,
            _ => return Err(anyhow!("unsupported sample format")),
        };
        Ok(stream)
    }

    fn writer_thread(rx: Receiver<AudioSample>, file_path: String) {
        let mut writer: Option<WavWriter<_>> = None;
        let mut frames = 0u64;

        while let Ok(chunk) = rx.recv() {
            if writer.is_none() {
                let spec = WavSpec {
                    channels: chunk.channels,
                    sample_rate: chunk.sample_rate,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                };
                writer = WavWriter::create(&file_path, spec).ok();
            }

            if let Some(w) = writer.as_mut() {
                for s in chunk.data {
                    if w.write_sample(s).is_err() {
                        eprintln!("WAV write error");
                        return;
                    }
                    frames += 1;
                }
            }
        }

        if let Some(w) = writer {
            if w.finalize().is_ok() {
                println!("audio saved → {file_path} ({frames} frames)");
            }
        }
    }
}
