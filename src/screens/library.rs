use ratatui::{
    prelude::*,
    widgets::*
};

use crate::{app::{
    App,
    }
};

use crate::library::{
    LibrarySelection,
    LibraryTrack,
    ArtistNode
};

use crate::library::VisibleRow;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let mut visible_rows = Vec::new();

    for (artist_index, artist) in app.library.artists.iter().enumerate() {
        visible_rows.push(VisibleRow::Artist { artist_index, artist });

        if artist.expanded {
            for (album_index, album) in artist.albums.iter().enumerate() {
                visible_rows.push(VisibleRow::Album {
                    artist_index,
                    album_index,
                    album_name: &album.name,
                });
            }
        }
    }

    let mut items = Vec::new();
    let mut selected_visual_index = 0;

    for (i, row) in visible_rows.iter().enumerate() {
        match row {
            VisibleRow::Artist { artist_index, artist } => {
                let selected = matches!(
                    app.library.selection,
                    Some(LibrarySelection::Artist { artist_index: ai }) if ai == *artist_index
                );
                if selected {
                    selected_visual_index = i;
                }
                let marker = if artist.expanded { "▾" } else { "▸" };
                let label = format!("{marker} {}", artist.name);
                let style = if selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                items.push(ListItem::new(label).style(style));
            }
            VisibleRow::Album { artist_index, album_index, album_name } => {
                let selected = matches!(
                    app.library.selection,
                    Some(LibrarySelection::Album {
                        artist_index: ai,
                        album_index: bi
                    }) if ai == *artist_index && bi == *album_index
                );
                if selected {
                    selected_visual_index = i;
                }
                let style = if selected {
                    Style::default().add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                items.push(ListItem::new(format!("  {album_name}")).style(style));
            }
        }
    }

    let mut list_state = ListState::default();
    list_state.select(Some(selected_visual_index));

    let list = List::new(items)
        .block(Block::default().title("Library").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Green).fg(Color::Black))
        .highlight_symbol("➤ ");

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // Draw right pane based on selection
    let tracks: Vec<&LibraryTrack> = match app.library.selection {
        Some(LibrarySelection::Artist { artist_index }) => app
            .library
            .artists
            .get(artist_index)
            .map(|artist| artist.albums.iter().flat_map(|a| &a.tracks).collect())
            .unwrap_or_default(),

        Some(LibrarySelection::Album {
            artist_index,
            album_index,
        }) => app
            .library
            .artists
            .get(artist_index)
            .and_then(|artist| artist.albums.get(album_index))
            .map(|album| album.tracks.iter().collect())
            .unwrap_or_default(),

        None => Vec::new(),
    };

    let track_items: Vec<ListItem> = tracks
        .iter()
        .map(|track| ListItem::new(track.title.clone()))
        .collect();

    let track_list = List::new(track_items)
        .block(Block::default().title("Tracks").borders(Borders::ALL));

    frame.render_widget(track_list, chunks[1]);
}
