mod app;
mod browser;
mod library;
mod list;
mod ui;
mod screens;

use app::{
    App,
    AppScreen
};

use crate::browser::BrowserItem;

use crate::library::scan_path_for_tracks;


use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{prelude::*, backend::CrosstermBackend};
use std::io::{stdout, Result};

fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('1') => app.goto_screen(app::AppScreen::Library),
                    KeyCode::Char('5') => app.goto_screen(app::AppScreen::Browser),
                    KeyCode::Char('a') => {
                        if app.screen == AppScreen::Browser {
                            if let Some(BrowserItem::Entry(path)) = app.browser.list.selected_item() {
                                let tracks = scan_path_for_tracks(path);
                                app.library.add_tracks(tracks);
                            }
                        }
                    }
                    KeyCode::Down => {
                        match app.screen {
                            AppScreen::Browser => app.browser.move_down(),
                            AppScreen::Library => app.library.move_down(),
                        }
                    }
                    KeyCode::Up => {
                        match app.screen {
                            AppScreen::Browser => app.browser.move_up(),
                            AppScreen::Library => app.library.move_up(),
                        }
                    }
                    KeyCode::Enter => {
                        if app.screen == AppScreen::Browser {
                            app.browser.open_selected();
                        }
                    }

                    KeyCode::Backspace => {
                        if app.screen == AppScreen::Browser {
                            app.browser.go_up();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
