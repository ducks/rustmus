mod app;
mod browser;
mod library;
mod list;
mod persistence;
mod player;
mod screens;
mod ui;

use app::{App, AppScreen};

use std::time::Instant;

use crate::browser::BrowserItem;

use crate::library::{LibraryFocus, scan_path_for_tracks};

use std::sync::Arc;
use std::sync::atomic::Ordering;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, prelude::*};
use std::io::{Result, stdout};

use simplelog::*;
use std::fs::File;

fn main() -> Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Trace,
        Config::default(),
        File::create("debug.log").unwrap(),
    )])
    .unwrap();

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        app.update();

        if app
            .player
            .lock()
            .unwrap()
            .autoplay_trigger
            .swap(false, Ordering::SeqCst)
        {
            let mut plyr = app.player.lock().unwrap();
            let mut lib = app.library.lock().unwrap();

            if let Some(current_path) = &plyr.current_path {
                if app.autoplay_enabled {
                    if let Some(next_path) = lib.next_track_path(current_path) {
                        lib.select_track_by_path(&next_path);
                        plyr.play(&next_path);

                        if let Some(next_track) = lib.track_by_path(&next_path) {
                            app.current_track = Some(next_track.clone());
                            app.playback_start = Some(Instant::now());

                            log::debug!(
                                "Autoplay switched to: {} – {}",
                                next_track.album_artist,
                                next_track.title
                            );
                            log::debug!("playback_start: {:?}", app.playback_start);
                        }
                    }
                }
            }
        }

        log::debug!("Drawing track: {:?}", app.current_track.as_ref().map(|t| &t.title));
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(200))? {

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('1') => app.goto_screen(app::AppScreen::Library),
                    KeyCode::Char('5') => app.goto_screen(app::AppScreen::Browser),
                    KeyCode::Char('a') => {
                        let mut lib = app.library_mut();

                        if app.screen == AppScreen::Browser {
                            if let Some(BrowserItem::Entry(path)) = app.browser.list.selected_item()
                            {
                                let tracks = scan_path_for_tracks(path);
                                lib.tracks = tracks.clone();

                                lib.add_tracks(tracks);
                            }
                        }
                    }

                    KeyCode::Down => match app.screen {
                        AppScreen::Browser => app.browser.move_down(),

                        AppScreen::Library => {
                            let mut lib = app.library_mut();

                            match lib.focus {
                                LibraryFocus::Left => lib.move_down(),
                                LibraryFocus::Right => {
                                    let count = lib.visible_tracks().len();
                                    lib.move_track_down(count);
                                }
                            }
                        },
                    },

                    KeyCode::Up => match app.screen {
                        AppScreen::Browser => app.browser.move_up(),

                        AppScreen::Library => {
                            let mut lib = app.library_mut();

                            match lib.focus {
                                LibraryFocus::Left => lib.move_up(),
                                LibraryFocus::Right => lib.move_track_up(),
                            }
                        },
                    },

                    KeyCode::Enter => {
                        if app.screen == AppScreen::Browser {
                            app.browser.open_selected();
                        }

                        let mut lib = app.library_mut();

                        if app.screen == AppScreen::Library
                            && lib.focus == LibraryFocus::Right
                        {
                            let selected = {
                                let lib = lib;
                                lib.visible_tracks().get(lib.track_index).cloned()
                            };

                            if let Some(track) = selected {
                                let player = Arc::clone(&app.player);

                                // Stop current playback and play selected track
                                {
                                    let mut plyr = player.lock().unwrap();
                                    plyr.stop();
                                    plyr.play(&track.path);

                                    app.current_track = Some(track.clone());
                                    app.playback_duration = track.duration.unwrap_or(0);
                                    app.playback_start = Some(Instant::now());
                                }
                            }
                        }
                    }

                    KeyCode::Char('p') => {
                        app.autoplay_enabled = !app.autoplay_enabled;
                    }

                    // KeyCode::Char('c') => {
                    //     if app.player.is_loaded() {
                    //         if app.player.is_paused() {
                    //             app.player.resume();
                    //         } else {
                    //             app.player.pause();
                    //         }
                    //     }
                    // }
                    KeyCode::Char('n') => {
                        app.play_next_track();
                    }

                    KeyCode::Backspace => {
                        if app.screen == AppScreen::Browser {
                            app.browser.go_up();
                        }
                    }
                    KeyCode::Char(' ') => {
                        let mut lib = app.library_mut();

                        if app.screen == AppScreen::Library {
                            lib.toggle_expanded();
                        }
                    }
                    KeyCode::Tab => {
                        let mut lib = app.library_mut();
                        lib.tab_focus();
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
