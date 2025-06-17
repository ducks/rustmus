use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::browser::BrowserState;

use crate::library::{
    LibraryState,
    LibraryTrack
};

use crate::persistence;

use crate::player::Player;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Library,
    Browser,
}

pub struct App {
    pub screen: AppScreen,
    pub browser: BrowserState,
    pub library: Arc<Mutex<LibraryState>>,
    pub player: Arc<Mutex<Player>>,
    pub play_queue: Vec<PathBuf>,
    pub queue_index: usize,
    pub autoplay_enabled: bool,
    pub current_track: Option<LibraryTrack>,

    /// Playback duration in seconds
    pub playback_duration: u64,

    /// Playback start
    pub playback_start: Option<Instant>,

    pub paused_at: Option<Instant>,
    pub paused_duration: Duration,
}

impl App {
    pub fn new() -> Self {
        let artists = persistence::load_library().unwrap_or_else(|_| vec![]);

        let library = Arc::new(Mutex::new(LibraryState::new()));
        library.lock().unwrap().artists = artists;
        library.lock().unwrap().rebuild_visible_rows(); // Make sure UI stays in sync

        Self {
            screen: AppScreen::Browser,
            browser: BrowserState::new(),
            library: library,
            player: Arc::new(Mutex::new(Player::new())),
            play_queue: Vec::new(),
            queue_index: 0,
            autoplay_enabled: true,
            current_track: None,
            playback_duration: 0,
            playback_start: None,
            paused_at: None,
            paused_duration: Duration::from_secs(0),
        }
    }

    pub fn player_mut(&self) -> std::sync::MutexGuard<'_, Player> {
        self.player.lock().unwrap()
    }

    pub fn library_mut(&self) -> std::sync::MutexGuard<'_, LibraryState> {
        self.library.lock().unwrap()
    }

    pub fn update(&mut self) {
        if self.autoplay_enabled
            && self.player_mut().is_loaded()
            && self.player_mut().is_done()
            && !self.player_mut().is_playing
        {
            self.play_next_track();
        }
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
        self.screen = screen
    }

    pub fn play_next_track(&mut self) {
        if self.queue_index + 1 < self.play_queue.len() {
            self.queue_index += 1;
            let next_path = self.play_queue[self.queue_index].clone();

           // Scope the library borrow once
            let next_track = {
                let mut lib = self.library_mut();
                lib.select_track_by_path(&next_path);
                lib.track_by_path(&next_path).cloned()
            };

            if let Some(track) = next_track {
                self.current_track = Some(track);
                self.playback_start = Some(Instant::now());
            } else {
                log::warn!("Could not find LibraryTrack for path: {:?}", next_path);
                self.current_track = None;
                self.playback_start = None;
            }

            self.player_mut().play(&next_path);
        } else {
            log::debug!("Reached end of queue");
            self.queue_index = 0;
            self.play_queue.clear();
            self.current_track = None;
            self.playback_start = None;
        }
    }


    pub fn set_play_queue(&mut self, tracks: Vec<PathBuf>, start_index: usize) {
        self.play_queue = tracks;
        self.queue_index = start_index;
    }

    pub fn pause(&mut self) {
        let mut player = self.player.lock().unwrap();
        player.set_paused(true);
        self.paused_at = Some(Instant::now());
    }

    pub fn resume(&mut self) {
        let mut player = self.player.lock().unwrap();
        player.set_paused(false);

        if let Some(paused_at) = self.paused_at {
            self.paused_duration += paused_at.elapsed();
        }

        self.paused_at = None;
    }

    pub fn toggle_pause(&mut self) {
        let is_paused = {
            let player = self.player.lock().unwrap();
            player.is_paused
        };

        if is_paused {
            self.resume();
        } else {
            self.pause();
        }
    }
}
