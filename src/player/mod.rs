mod decoder;
mod output;

use crate::player::thread::JoinHandle;

use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use symphonia::core::{
    audio::{ AudioBufferRef, Signal },
    codecs::{ CODEC_TYPE_NULL, DecoderOptions },
    formats::{FormatOptions},
    io::MediaSourceStream,
    meta::MetadataOptions,
};

use symphonia::default::{ get_codecs, get_probe };

use log::{info, debug, error};

use std::collections::VecDeque;



pub struct Player {
    pub current_path: Option<PathBuf>,
    pub is_playing: bool,
    pub autoplay: bool,
    pub handle: Option<JoinHandle<()>>,
    stream: Option<Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            current_path: None,
            is_playing: false,
            autoplay: true,
            stream: None,
            handle: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn play(&mut self, path: &Path) {
        self.stop(); // Stop any current playback

        let file = File::open(path).expect("Failed to open file");
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let probed = get_probe()
            .format(
                &Default::default(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("Unsupported format");

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("No supported audio track found");

        let mut decoder = get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .expect("Unsupported codec");

        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.unwrap().count();

        // Create CPAL output stream
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device available");

        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = Arc::clone(&buffer);

        let sample_buf = Arc::new(Mutex::new(VecDeque::<f32>::new()));
        let sample_buf_clone = Arc::clone(&sample_buf);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    let mut buf = sample_buf_clone.lock().unwrap();

                    for sample in data.iter_mut() {
                        *sample = buf.pop_front().unwrap_or(0.0); // Pop from front = correct order
                    }
                },
                move |err| eprintln!("CPAL stream error: {err}"),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play stream");

        self.is_playing = true;
        self.current_path = Some(path.to_path_buf());

        // Spawn decoding thread
        let decode_buffer = Arc::clone(&sample_buf);
        let handle = thread::spawn(move || {
            while let Ok(packet) = format.next_packet() {
                let decoded = match decoder.decode(&packet) {
                    Ok(decoded) => decoded,
                    Err(err) => {
                        eprintln!("Decode error: {err}");
                        continue;
                    }
                };

                let spec = decoded.spec();
                log::debug!("Decoded: sample_rate={}, channels={}",  spec.rate, spec.channels.count());
                log::debug!("CPAL: sample_rate={}, channels={}", config.sample_rate.0, config.channels);

                let mut samples = Vec::new();

                match &decoded {
                    AudioBufferRef::F32(_) => log::debug!("Decoded buffer format: F32"),
                    AudioBufferRef::S16(_) => log::debug!("Decoded buffer format: S16"),
                    AudioBufferRef::U8(_)  => log::debug!("Decoded buffer format: U8"),
                    AudioBufferRef::S24(_) => log::debug!("Decoded buffer format: S24"),
                    _ => log::debug!("Decoded buffer format: Unknown/Unsupported"),
                }

                match decoded {
                    AudioBufferRef::F32(buf) => {
                        for frame in 0..buf.frames() {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[frame]);
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        for frame in 0..buf.frames() {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[frame] as f32 / i16::MAX as f32);
                            }
                        }
                    }
                    AudioBufferRef::U8(buf) => {
                        for frame in 0..buf.frames() {
                            for ch in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(ch)[frame] as f32 / u8::MAX as f32);
                            }
                        }
                    }
                    // AudioBufferRef::S24(buf) => {
                    //     for frame in 0..buf.frames() {
                    //         for chan in 0..buf.spec().channels.count() {
                    //             let sample = buf.chan(chan)[frame].into();
                    //             samples.push(sample as f32 / (1 << 23) as f32);
                    //         }
                    //     }
                    // }
                    AudioBufferRef::F64(buf) => {
                        for frame in 0..buf.frames() {
                            for chan in 0..buf.spec().channels.count() {
                                samples.push(buf.chan(chan)[frame] as f32);
                            }
                        }
                    }
                    _ => {
                        eprintln!("Unsupported buffer format");
                        continue;
                    }
                }

                decode_buffer.lock().unwrap().extend(samples);
                std::thread::sleep(Duration::from_millis(10));
            }

            eprintln!("Finished decoding.");
        });

        self.handle = Some(handle);
        self.stream = Some(stream); // store the stream if needed for later stop/resume
        self.buffer = buffer;
    }


    pub fn stop(&mut self) {
        self.stream = None;
        self.is_playing = false;
        self.current_path = None;
        self.buffer.lock().unwrap().clear();
    }

    pub fn is_loaded(&self) -> bool {
        self.current_path.is_some()
    }

    pub fn is_done(&self) -> bool {
        self.buffer.lock().unwrap().is_empty() && self.is_playing
    }
}
