use crate::{
    app::{App, AppScreen},
    screens,
};
use ratatui::prelude::*;

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    match app.screen {
        AppScreen::Library => screens::library::draw(frame, area, app),
        AppScreen::Browser => screens::browser::draw(frame, area, app),
    }
}

pub fn highlight_style(screen: AppScreen) -> Style {
    match screen {
        AppScreen::Library => Style::default().bg(Color::Green).fg(Color::Black),
        AppScreen::Browser => Style::default().bg(Color::Blue).fg(Color::White),
    }
}
