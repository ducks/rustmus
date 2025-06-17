use crate::{
    app::{App, AppScreen},
    screens,
};
use ratatui::prelude::*;
use ratatui::widgets::{
    Gauge,
    Paragraph,
};

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main screen
            Constraint::Length(2), // Footer
        ])
        .split(frame.area());

    match app.screen {
        AppScreen::Library => screens::library::draw(frame, app, layout[0]),
        AppScreen::Browser => screens::browser::draw(frame, app, layout[0]),
        _ => println!("ok"),
    }

    render_footer(frame, app, layout[1]);
}

pub fn highlight_style(screen: AppScreen) -> Style {
    match screen {
        AppScreen::Library => Style::default().bg(Color::Green).fg(Color::Black),
        AppScreen::Browser => Style::default().bg(Color::Blue).fg(Color::White),
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    if let Some(track) = &app.current_track {
        log::debug!(
            "Rendering footer: {} – {}, elapsed: {:?}",
            track.album_artist,
            track.title,
            app.playback_start.map(|t| t.elapsed())
        );

        let elapsed = app.playback_start
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0);

        let dur = track.duration.unwrap_or(0);
        let pos = elapsed.min(dur);

        let percent = if dur > 0 {
            pos as f64 / dur as f64
        } else {
            0.0
        };

        let info_line = Paragraph::new(format!(
            "▶ {} – {}  {:02}:{:02} / {:02}:{:02}",
            track.album_artist,
            track.title,
            pos / 60, pos % 60,
            dur / 60, dur % 60,
        ))
        .style(Style::default().fg(Color::Gray));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        let info_chunk = chunks[0];
        let gauge_chunk = chunks[1];

        f.render_widget(info_line, info_chunk);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::LightGreen))
            .ratio(percent);

        f.render_widget(gauge, gauge_chunk);
    } else {
        let empty = Paragraph::new("⏹ Nothing playing")
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(empty, area);
    }
}

