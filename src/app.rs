use crate::browser::BrowserState;

use crate::library::{LibraryFocus, LibraryState};

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
    pub library: LibraryState,
    pub player: Player,
}

impl App {
    pub fn new() -> Self {
        let artists = persistence::load_library().unwrap_or_else(|_| vec![]);

        let mut library = LibraryState::new();
        library.artists = artists;
        library.rebuild_visible_rows(); // Make sure UI stays in sync

        Self {
            screen: AppScreen::Library,
            browser: BrowserState::new(),
            library: library,
            player: Player::new(),
        }
    }

    pub fn update(&mut self) {
        if self.player.autoplay && self.player.is_loaded() && self.player.is_done() {
            self.play_next_track();
        }
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
        self.screen = screen
    }

    pub fn play_next_track(&mut self) {
        eprintln!("called play_next_track");

        if let Some(current) = &self.player.current_path {
            eprintln!("currently playing: {:?}", current);

            if let Some(next_path) = self.library.next_track_path(current) {
                eprintln!("next track: {:?}", next_path);
                self.library.select_track_by_path(&next_path);
                self.player.play(&next_path);
            } else {
                eprintln!("no next track found");
            }
        } else {
            eprintln!("no currently playing track");
        }
    }
}
