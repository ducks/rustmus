use crate::browser::{
    BrowserState
};

use crate::library::LibraryState;

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
        Self {
            screen: AppScreen::Library,
            browser: BrowserState::new(),
            library: LibraryState::new(),
        }
    }

    pub fn goto_screen(&mut self, screen: AppScreen) {
         self.screen = screen
    }
}
