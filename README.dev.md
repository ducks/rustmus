# Developer Guide: rustmus

This document is for contributors and maintainers of `rustmus`.
For user-facing instructions, see [README.md](./README.md).

---

## Project Structure

```text
src/
├── app.rs          # Core state and logic for App, Screens, Player
├── browser.rs      # Directory browsing logic
├── library.rs      # Library view, album/artist/track state
├── player.rs       # MP3 playback using rodio
├── screens/        # UI rendering per screen
├── ui.rs           # Layout split, keybindings, etc.
├── persistence.rs  # JSON load/save for artist + track library
```

## Key Features & State

- `App` tracks the active screen (`Library`, `Browser`, etc.)
- Library state is split into:
  - Left pane: artists and albums (with expandable tree)
  - Right pane: visible tracks, selectable with Tab
- Tracks are tagged using `id3` crate on `.mp3` files
- Playback uses `rodio::Sink` with a manual autoplay toggle

---

## Keybindings

| Key           | Action                          |
|---------------|---------------------------------|
| `1`           | Go to Library                   |
| `5`           | Go to Browser                   |


### Browser View

| Key           | Action                          |
|---------------|---------------------------------|
| `a`           | Add file/dir to playlist        |

### Library View

| Key           | Action                          |
|---------------|---------------------------------|
| `Tab`         | Toggle focus left/right         |
| `Enter`       | Play selected track             |
| `c`           | Toggle pause/resume             |
| `n`           | Next song                       |
| `p`           | Toggle autoplay                 |
| `Backspace`   | Go up a directory (Browser)     |
| `Space`       | Toggle artist/album view        |
| `Up/Down`     | Navigate lists                  |

---

## Persistence

- Library state is saved to `library.json`
- Autoloaded during `App::new()` if it exists
- Only `.mp3` files are supported for now
- Duplicates are ignored silently

---

## Future Todos

- Add `flac` support via `symphonia` or similar
- Implement seeking via Symphonia → rodio integration
- Support playlist screen and queue
- Playback progress bar and time display
- Configurable keybindings
- Better error handling/logging
- Autplay

---

## Contributing

Pull requests welcome! Please:
- Run `cargo fmt`
- Ensure it builds with `cargo check`
- Test playback, UI, and persistence
