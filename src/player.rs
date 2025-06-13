use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct Player {
    _stream: OutputStream, // needs to live as long as Sink
    handle: OutputStreamHandle,
    sink: Sink,
    pub current_path: Option<PathBuf>,
    is_paused: bool,
    pub autoplay: bool,
}

impl Player {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();

        Self {
            _stream,
            handle,
            sink,
            is_paused: false,
            current_path: None,
            autoplay: true,
        }
    }

    pub fn play<P: AsRef<Path>>(&mut self, path: P) {
        let new_path = path.as_ref().to_path_buf();

        // Only create a new sink if this is a new file
        if self.current_path.as_ref() != Some(&new_path) {
            self.stop();

            let file = File::open(&new_path).unwrap();
            let source = Decoder::new(BufReader::new(file)).unwrap();

            let (_stream, handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&handle).unwrap();
            sink.append(source);

            self._stream = _stream;
            self.sink = sink;
            self.current_path = Some(new_path);
            self.is_paused = false;
        } else {
            // Already loaded — just resume
            self.resume();
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn pause(&mut self) {
        if !self.is_paused {
            self.sink.pause();
            self.is_paused = true;
        }
    }

    pub fn resume(&mut self) {
        if self.is_paused {
            self.sink.play();
            self.is_paused = false;
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn is_playing(&self) -> bool {
        !self.is_paused && !self.sink.empty()
    }

    pub fn is_loaded(&self) -> bool {
        !self.sink.empty() || self.is_paused
    }

    pub fn is_done(&self) -> bool {
        self.sink.empty() && !self.is_paused
    }
}
