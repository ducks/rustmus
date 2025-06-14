use id3::Tag;
use ratatui::widgets::{ListItem, ListState};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::persistence;

#[derive(Debug, Clone)]
pub enum VisibleRow {
    Artist {
        artist_index: usize,
    },
    Album {
        artist_index: usize,
        album_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibrarySelection {
    Artist {
        artist_index: usize,
    },
    Album {
        artist_index: usize,
        album_index: usize,
    },
}

pub struct LibraryState {
    pub artists: Vec<ArtistNode>,
    pub selection: Option<LibrarySelection>,
    pub state: ListState,
    pub album_index: Option<usize>,
    pub focus: LibraryFocus,
    pub track_index: usize,
    pub visible_rows: Vec<VisibleRow>,
}

impl LibraryState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            artists: Vec::new(),
            selection: Some(LibrarySelection::Artist { artist_index: 0 }),
            state,
            album_index: Some(0),
            focus: LibraryFocus::Left,
            track_index: 0,
            visible_rows: Vec::new(),
        }
    }

    pub fn add_tracks(&mut self, tracks: Vec<LibraryTrack>) {
        for track in tracks {
            // Check if artist exists
            if let Some(artist) = self.artists.iter_mut().find(|a| a.name == track.artist) {
                // Check if album exists
                if let Some(album) = artist.albums.iter_mut().find(|a| a.name == track.album) {
                    // Check for duplicate by path
                    let already_exists = album.tracks.iter().any(|t| t.path == track.path);
                    if !already_exists {
                        album.tracks.push(track);
                        album.tracks.sort_by_key(|t| t.track_number.unwrap_or(999));
                    }
                } else {
                    artist.albums.push(AlbumNode {
                        name: track.album.clone(),
                        tracks: vec![track],
                    });
                }
            } else {
                self.artists.push(ArtistNode {
                    name: track.artist.clone(),
                    albums: vec![AlbumNode {
                        name: track.album.clone(),
                        tracks: vec![track],
                    }],
                    expanded: false,
                });
            }
        }

        self.artists.sort_by_key(|a| a.name.clone());

        self.rebuild_visible_rows(); // <-- Important

        // Optional: auto-select the first row
        if self.selection.is_none() && !self.visible_rows.is_empty() {
            self.selection = Some(Self::row_to_selection(&self.visible_rows[0]));
            self.state.select(Some(0));
        }

        persistence::save_library(&self.artists).ok();
    }

    pub fn move_down(&mut self) {
        let visual_rows = Self::build_visible_rows(&self.artists);

        let current_index = Self::selected_index(&visual_rows, self.selection);
        let next_index = (current_index + 1).min(visual_rows.len().saturating_sub(1));
        self.selection = visual_rows.get(next_index).map(Self::row_to_selection);
    }

    pub fn move_up(&mut self) {
        let visual_rows = Self::build_visible_rows(&self.artists);

        let current_index = Self::selected_index(&visual_rows, self.selection);
        let next_index = current_index.saturating_sub(1);
        self.selection = visual_rows.get(next_index).map(Self::row_to_selection);
    }

    pub fn toggle_expanded(&mut self) {
        if let Some(LibrarySelection::Artist { artist_index }) = self.selection {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                artist.expanded = !artist.expanded;
                self.rebuild_visible_rows();
            }
        }
    }

    pub fn selected_artist(&self) -> Option<&ArtistNode> {
        match self.selection {
            Some(LibrarySelection::Artist { artist_index }) => self.artists.get(artist_index),
            Some(LibrarySelection::Album { artist_index, .. }) => self.artists.get(artist_index),
            None => None,
        }
    }

    pub fn selected_album(&self) -> Option<&AlbumNode> {
        match self.selection {
            Some(LibrarySelection::Album {
                artist_index,
                album_index,
            }) => self.artists.get(artist_index)?.albums.get(album_index),
            _ => None,
        }
    }

    fn build_visible_rows<'a>(artists: &'a [ArtistNode]) -> Vec<VisibleRow> {
        let mut rows = Vec::new();
        for (artist_index, artist) in artists.iter().enumerate() {
            rows.push(VisibleRow::Artist { artist_index });
            if artist.expanded {
                for (album_index, album) in artist.albums.iter().enumerate() {
                    rows.push(VisibleRow::Album {
                        artist_index,
                        album_index,
                    });
                }
            }
        }
        rows
    }

    fn selected_index<'a>(rows: &'a [VisibleRow], selection: Option<LibrarySelection>) -> usize {
        rows.iter()
            .position(|row| match (row, selection) {
                (
                    VisibleRow::Artist { artist_index, .. },
                    Some(LibrarySelection::Artist { artist_index: ai }),
                ) => *artist_index == ai,
                (
                    VisibleRow::Album {
                        artist_index,
                        album_index,
                        ..
                    },
                    Some(LibrarySelection::Album {
                        artist_index: ai,
                        album_index: bi,
                    }),
                ) => *artist_index == ai && *album_index == bi,
                _ => false,
            })
            .unwrap_or(0)
    }

    pub fn row_to_selection(row: &VisibleRow) -> LibrarySelection {
        match row {
            VisibleRow::Artist { artist_index } => LibrarySelection::Artist {
                artist_index: *artist_index,
            },
            VisibleRow::Album {
                artist_index,
                album_index,
            } => LibrarySelection::Album {
                artist_index: *artist_index,
                album_index: *album_index,
            },
        }
    }

    pub fn tab_focus(&mut self) {
        self.focus = match self.focus {
            LibraryFocus::Left => LibraryFocus::Right,
            LibraryFocus::Right => LibraryFocus::Left,
        };
    }

    pub fn move_track_up(&mut self) {
        if self.track_index > 0 {
            self.track_index -= 1;
        }
    }

    pub fn move_track_down(&mut self, track_count: usize) {
        if self.track_index + 1 < track_count {
            self.track_index += 1;
        }
    }

    pub fn visible_tracks(&self) -> Vec<&LibraryTrack> {
        match self.selection {
            Some(LibrarySelection::Artist { artist_index }) => self
                .artists
                .get(artist_index)
                .map(|a| a.albums.iter().flat_map(|alb| &alb.tracks).collect())
                .unwrap_or_default(),
            Some(LibrarySelection::Album {
                artist_index,
                album_index,
            }) => self
                .artists
                .get(artist_index)
                .and_then(|a| a.albums.get(album_index))
                .map(|alb| alb.tracks.iter().collect())
                .unwrap_or_default(),
            None => vec![],
        }
    }

    pub fn rebuild_visible_rows(&mut self) {
        self.visible_rows.clear();

        for (artist_index, artist) in self.artists.iter().enumerate() {
            self.visible_rows.push(VisibleRow::Artist { artist_index });

            if artist.expanded {
                for (album_index, _) in artist.albums.iter().enumerate() {
                    self.visible_rows.push(VisibleRow::Album {
                        artist_index,
                        album_index,
                    });
                }
            }
        }

        // Restore selection if it was valid
        if self.visible_rows.is_empty() {
            self.selection = None;
            self.state.select(None);
        } else {
            // Default to first item if selection missing or invalid
            let current_index = self
                .visible_rows
                .iter()
                .position(|row| Some(Self::row_to_selection(row)) == self.selection)
                .unwrap_or(0);

            self.selection = Some(Self::row_to_selection(&self.visible_rows[current_index]));
            self.state.select(Some(current_index));
        }
    }

    pub fn right_pane_items(&self) -> (Vec<ListItem>, Vec<usize>) {
        let tracks = self.visible_tracks();
        let mut items = Vec::new();
        let mut playable_indices = Vec::new();
        let mut last_album: Option<&str> = None;

        for track in tracks {
            let album = track.album.as_str();
            if last_album != Some(album) {
                items.push(ListItem::new(format!("{}:", album)));
                last_album = Some(album);
            }

            playable_indices.push(items.len()); // index where this track will be
            let number = track
                .track_number
                .map_or("--".to_string(), |n| format!("{:02}", n));
            items.push(ListItem::new(format!("  {}. {}", number, track.title)));
        }

        (items, playable_indices)
    }

    pub fn next_track_path(&self, current: &Path) -> Option<PathBuf> {
        let tracks = self.visible_tracks();

        for (i, track) in tracks.iter().enumerate() {
            if track.path == current && i + 1 < tracks.len() {
                return Some(tracks[i + 1].path.clone());
            }
        }

        None
    }

    pub fn select_track_by_path(&mut self, path: &Path) {
        let tracks = self.visible_tracks();
        if let Some(i) = tracks.iter().position(|t| &t.path == path) {
            self.track_index = i;
            self.state.select(Some(i));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryTrack {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumNode {
    pub name: String,
    pub tracks: Vec<LibraryTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistNode {
    pub name: String,
    pub albums: Vec<AlbumNode>,
    pub expanded: bool,
}

pub fn scan_path_for_tracks(path: &Path) -> Vec<LibraryTrack> {
    let mut tracks = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.eq_ignore_ascii_case("mp3") || s.eq_ignore_ascii_case("flac"))
            != Some(true)
        {
            continue;
        }

        let tag = Tag::read_from_path(path).ok();
        let title = tag
            .as_ref()
            .and_then(|t| t.title())
            .unwrap_or("Unknown Title")
            .to_string();
        let artist = tag
            .as_ref()
            .and_then(|t| t.artist())
            .unwrap_or("Unknown Artist")
            .to_string();
        let album = tag
            .as_ref()
            .and_then(|t| t.album())
            .unwrap_or("Unknown Album")
            .to_string();
        let track_number = tag.as_ref().and_then(|t| t.track()).map(|n| n as u32);

        tracks.push(LibraryTrack {
            path: path.to_path_buf(),
            title,
            artist,
            album,
            track_number,
        });
    }

    tracks
}

#[derive(PartialEq)]
pub enum LibraryFocus {
    Left,
    Right,
}
