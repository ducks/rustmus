use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct AudioOutput {
    pub sample_rate: u32,
    pub channels: u16,
    stream: Option<cpal::Stream>,
    pub buffer: Arc<Mutex<Vec<f32>>>, // Interleaved stereo samples
}

impl AudioOutput {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device found"))?;

        let config = device.default_output_config()?.config();

        let channels = self.channels as usize;
        let buffer = self.buffer.clone();

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let mut buf = buffer.lock().unwrap();
                for frame in data.chunks_mut(channels) {
                    for sample in frame.iter_mut() {
                        *sample = buf.pop().unwrap_or(0.0);
                    }
                }
            },
            move |err| {
                eprintln!("CPAL stream error: {err}");
            },
            None
        )?;

        stream.play()?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Append decoded PCM samples to buffer.
    pub fn push_samples(&mut self, samples: &[f32]) {
        let mut buf = self.buffer.lock().unwrap();
        buf.extend_from_slice(samples);
    }
}
