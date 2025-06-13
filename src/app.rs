use crate::browser::{
    BrowserState
};

use crate::library::LibraryState;

use crate::persistence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Library,
    Browser,
}

pub struct App {
    pub screen: AppScreen,
    pub browser: BrowserState,
    pub library: LibraryState
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
        }
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
         self.screen = screen
    }
}
