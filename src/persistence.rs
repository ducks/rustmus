use std::{fs, path::Path};
use crate::library::ArtistNode;

const SAVE_PATH: &str = "library.json"; // or "library.ron"

pub fn save_library(artists: &[ArtistNode]) -> std::io::Result<()> {
    let data = serde_json::to_string_pretty(artists)?;
    fs::write(SAVE_PATH, data)?;
    Ok(())
}

pub fn load_library() -> std::io::Result<Vec<ArtistNode>> {
    if Path::new(SAVE_PATH).exists() {
        let data = fs::read_to_string(SAVE_PATH)?;
        let artists = serde_json::from_str(&data)?;
        Ok(artists)
    } else {
        Ok(vec![]) // start empty if no file
    }
}
