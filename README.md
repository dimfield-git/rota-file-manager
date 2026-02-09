# Rota File Manager

A keyboard-driven TUI file explorer for TedOS, built in Rust with [ratatui](https://github.com/ratatui/ratatui) and [crossterm](https://github.com/crossterm-rs/crossterm).

Rota is currently in **Phase 0** — a read-only directory browser that establishes the foundation for a full file manager. It can navigate the filesystem, display directory listings, and show file metadata, but does not yet modify any files.

---

## Features

- Two-panel layout: file list on the left, details on the right
- Directory-first sorting with case-insensitive alphabetical ordering
- Human-readable file sizes (B / KiB / MiB / GiB)
- Vim-style keyboard navigation
- Errors surfaced in the UI instead of crashing
- Clean terminal lifecycle (raw mode, alternate screen, guaranteed teardown)

## Keybindings

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Open selected directory |
| `Backspace` | Go to parent directory |
| `r` | Refresh directory listing |
| `q` | Quit |

## Building

Requires Rust 2024 edition (1.85+).

```sh
cargo build --release
```

The binary will be at `target/release/rota-file-manager`.

## Running

```sh
# Run from the current directory
cargo run

# Or run the built binary directly
./target/release/rota-file-manager
```

## Project Structure

```
src/
└── main.rs          # Application state, UI rendering, input handling, and entry point
docs/
├── ROADMAP.md       # Full development roadmap (Phase 0–8)
└── rota_main_rs_description.md  # Detailed walkthrough of main.rs
```

## Dependencies

| Crate | Purpose |
|---|---|
| `ratatui` 0.28 | Terminal UI framework (widgets, layout, rendering) |
| `crossterm` 0.28 | Cross-platform terminal manipulation and input |

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for the full plan. The next milestones are:

- **Phase 1** — CLI arguments, timestamp formatting, hidden file toggle, sort modes, better scrolling
- **Phase 2** — Incremental search/filter, text file preview, directory size calculation
- **Phase 3** — File operations (copy, move, delete, rename) behind confirmation prompts

## License

[MIT](LICENSE)
