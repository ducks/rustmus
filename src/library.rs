use std::path::{ Path, PathBuf };
use walkdir::WalkDir;
use id3::{ Tag, TagLike };
use ratatui::widgets::ListState;

pub enum VisibleRow<'a> {
    Artist {
        artist_index: usize,
        artist: &'a ArtistNode,
    },
    Album {
        artist_index: usize,
        album_index: usize,
        album_name: &'a str,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum LibrarySelection {
    Artist { artist_index: usize },
    Album { artist_index: usize, album_index: usize },
}

pub struct LibraryState {
    pub artists: Vec<ArtistNode>,
    pub selection: Option<LibrarySelection>,
    pub state: ListState,
}

impl LibraryState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            artists: Vec::new(),
            selection: Some(LibrarySelection::Artist { artist_index: 0 }),
            state,
        }
    }

    pub fn add_tracks(&mut self, tracks: Vec<LibraryTrack>) {
        for track in tracks {
            let artist_entry = self.artists.iter_mut()
                .find(|a| a.name == track.artist);

            if let Some(artist) = artist_entry {
                if let Some(album) = artist.albums.iter_mut().find(|a| a.name == track.album) {
                    album.tracks.push(track);
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
    }

    pub fn move_down(&mut self) {
        let mut visual_rows = Self::build_visible_rows(&self.artists);

        let current_index = Self::selected_index(&visual_rows, self.selection);
        let next_index = (current_index + 1).min(visual_rows.len().saturating_sub(1));
        self.selection = visual_rows.get(next_index).map(Self::row_to_selection);
    }

    pub fn move_up(&mut self) {
        let mut visual_rows = Self::build_visible_rows(&self.artists);

        let current_index = Self::selected_index(&visual_rows, self.selection);
        let next_index = current_index.saturating_sub(1);
        self.selection = visual_rows.get(next_index).map(Self::row_to_selection);
    }

    pub fn toggle_expanded(&mut self) {
        if let Some(LibrarySelection::Artist { artist_index }) = self.selection {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                artist.expanded = !artist.expanded;
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

    fn build_visible_rows<'a>(artists: &'a [ArtistNode]) -> Vec<VisibleRow<'a>> {
        let mut rows = Vec::new();
        for (artist_index, artist) in artists.iter().enumerate() {
            rows.push(VisibleRow::Artist { artist_index, artist });
            if artist.expanded {
                for (album_index, album) in artist.albums.iter().enumerate() {
                    rows.push(VisibleRow::Album {
                        artist_index,
                        album_index,
                        album_name: &album.name,
                    });
                }
            }
        }
        rows
    }

    fn selected_index<'a>(rows: &'a [VisibleRow], selection: Option<LibrarySelection>) -> usize {
        rows.iter()
            .position(|row| match (row, selection) {
                (VisibleRow::Artist { artist_index, .. }, Some(LibrarySelection::Artist { artist_index: ai })) => *artist_index == ai,
                (VisibleRow::Album { artist_index, album_index, .. }, Some(LibrarySelection::Album { artist_index: ai, album_index: bi })) => *artist_index == ai && *album_index == bi,
                _ => false,
            })
        .unwrap_or(0)
    }

    fn row_to_selection(row: &VisibleRow) -> LibrarySelection {
        match row {
            VisibleRow::Artist { artist_index, .. } => LibrarySelection::Artist { artist_index: *artist_index },
            VisibleRow::Album { artist_index, album_index, .. } => LibrarySelection::Album {
                artist_index: *artist_index,
                album_index: *album_index,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LibraryTrack {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
}

#[derive(Debug, Clone)]
pub struct AlbumNode {
    pub name: String,
    pub tracks: Vec<LibraryTrack>,
}

#[derive(Debug, Clone)]
pub struct ArtistNode {
    pub name: String,
    pub albums: Vec<AlbumNode>,
    pub expanded: bool,
}

pub fn scan_path_for_tracks(path: &Path) -> Vec<LibraryTrack> {
    let mut tracks = Vec::new();

    let walker: Box<dyn Iterator<Item = PathBuf>> = if path.is_dir() {
        Box::new(
            WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .map(|e| e.path().to_path_buf())
        )
    } else {
        Box::new(std::iter::once(path.to_path_buf()))
    };

    for path in walker.filter(|p| p.extension().map(|ext| ext == "mp3").unwrap_or(false)) {
        let tag = Tag::read_from_path(&path).ok();
        let title = tag.as_ref().and_then(|t| t.title()).unwrap_or("Unknown Title").to_string();
        let artist = tag.as_ref().and_then(|t| t.artist()).unwrap_or("Unknown Artist").to_string();
        let album = tag.as_ref().and_then(|t| t.album()).unwrap_or("Unknown Album").to_string();

        tracks.push(LibraryTrack {
            path,
            title,
            artist,
            album,
        });
    }

    tracks
}

