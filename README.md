# TUI Music Player (Rust)

A terminal-based music player written in Rust using
[ratatui](https://github.com/ratatui-org/ratatui) and
[crossterm](https://github.com/crossterm-rs/crossterm). Navigate your
filesystem, browse audio files, and play music — all in the terminal.

## Features

- TUI interface with multiple screens
  - Library view
  - File browser
  - Playlist (coming soon)
- Navigate directories using keyboard
- Dotfiles are hidden by default
- Modular, extensible codebase

## Screenshots

_Coming soon..._

## Installation

```bash
git clone https://github.com/yourname/tui-music-player.git
cd tui-music-player
cargo run
```

## Controls

| Key              | Action                      |
|------------------|-----------------------------|
| `1`              | Go to Library Screen        |
| `5`              | Go to Browser Screen        |
| `↑ / ↓`          | Move selection              |
| `Enter`          | Enter directory / open item |
| `q`              | Quit                        |

## Planned Features / TODO

- [x] Hide dotfiles
- [ ] Sort directories before files
- [ ] Audio playback via `rodio`
- [ ] Playlist screen with queue
- [ ] Footer bar with help / now playing
- [ ] Config file for keybindings and paths
- [ ] Save/restore last visited directory
- [ ] Match more `cmus` keybindings and behaviors (e.g. `Tab`, `v`, `:`)

