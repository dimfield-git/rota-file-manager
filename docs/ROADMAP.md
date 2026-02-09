# Rota File Manager — Roadmap

> **Goal:** A fully functioning, keyboard-driven TUI file manager built in Rust with crossterm + ratatui.

---

## Current State (Phase 0 — Complete)

- Read-only directory listing with two-panel layout (file list + details)
- Navigation: `j`/`k`, arrows, `Enter` (open dir), `Backspace` (parent), `r` (refresh), `q` (quit)
- Directories sorted first, then case-insensitive alphabetical
- Human-readable file sizes (B / KiB / MiB / GiB)
- Errors surfaced in the UI footer instead of panicking
- Terminal lifecycle handled cleanly (raw mode, alternate screen, guaranteed teardown)

---

## Phase 1 — Polish the Read-Only Foundation

Small, low-risk improvements that don't touch file operations. Each is independently mergeable.

### 1.1 CLI arguments

- [ ] Accept `--path <dir>` to start in a specific directory (default: cwd)
- [ ] Accept `--show-hidden` flag (or default-off, toggled at runtime)
- **Crate choice:** `clap` (feature-rich, derive macro) **or** `lexopt` (zero-dependency, minimal)

### 1.2 Format modified time

- [ ] Display human-readable timestamps in the details panel (e.g. `2025-06-14 09:32`)
- **Crate choice:** `chrono` (mature, heavy) **or** `time` (lighter, `#![no_std]`-friendly) **or** roll your own with `SystemTime::duration_since(UNIX_EPOCH)` + manual formatting

### 1.3 Hidden file toggle

- [ ] Press `.` (or `h`) to toggle visibility of dotfiles
- [ ] Persist preference via CLI flag or config (see Phase 5)

### 1.4 Sorting options

- [ ] Cycle through sort modes with `s`: name → size → modified → extension
- [ ] Show current sort mode in the header or footer
- [ ] Reverse sort with `S` (or `shift+s`)

### 1.5 Scrolling and viewport

- [ ] Page-up / page-down (`Ctrl+u` / `Ctrl+d`, or `PgUp` / `PgDn`)
- [ ] Home / End to jump to first / last entry
- [ ] `G` / `gg` (vim-style) as alternatives

### 1.6 Symlink awareness

- [ ] Display symlink targets in the details panel
- [ ] Distinguish symlinks visually (e.g. `[LNK]` prefix or different style)
- [ ] Follow vs. show link metadata option

---

## Phase 2 — Search, Filter, and Preview

These features keep the manager read-only but dramatically improve usability.

### 2.1 Incremental filter / search

- [ ] Press `/` to enter filter mode; typed characters narrow the visible list in real-time
- [ ] `Esc` to cancel, `Enter` to accept and jump to first match
- [ ] Highlight matching substring in each entry name

### 2.2 Text file preview pane

- [ ] Replace (or augment) the details panel with a file preview when a text file is selected
- [ ] Show first N lines of `.txt`, `.md`, `.rs`, `.toml`, `.json`, etc.
- [ ] Detect binary files and display "(binary)" or hex dump snippet
- **Layout choice:**
  - **A) Replace details panel** — simpler; toggle with `p` between details and preview
  - **B) Third vertical panel** — miller-columns style (tree | list | preview); more complex layout math

### 2.3 Directory size calculation

- [ ] On-demand (`d` key) recursive size calculation for the selected directory
- [ ] Run in a background thread; show spinner or "calculating…" in the details panel
- [ ] Cache results for the session

### 2.4 Breadcrumb / path bar

- [ ] Show clickable-looking breadcrumb path in the header (e.g. `/ > home > dim > projects > rota`)
- [ ] Allow jumping to any ancestor by keybinding (e.g. number keys or a quick-select popup)

---

## Phase 3 — File Operations (Core)

This is the transition from "viewer" to "manager." Every destructive operation must be behind a confirmation prompt.

### 3.1 Architecture: command & confirmation layer

- [ ] Introduce a `Command` enum representing pending operations
- [ ] Modal confirmation dialog widget ("Delete 3 files? [y/N]")
- [ ] Operation result feedback in the status bar ("Copied 2 files", "Error: permission denied")
- [ ] Consider an undo stack (see Phase 6)

### 3.2 Selection model

- [ ] Space to toggle-select the current entry (visual mark, e.g. `*` prefix or highlight color)
- [ ] `v` for visual/range select
- [ ] `a` to select all / deselect all
- [ ] Selection count shown in the footer

### 3.3 Copy and move

- [ ] `y` (yank) to mark selected entries for copy
- [ ] `x` (cut) to mark selected entries for move
- [ ] `p` (paste) to execute in the current directory
- [ ] Progress indicator for large operations (file count / bytes)
- **Implementation choice:**
  - **A) Blocking in main thread** — simplest; UI freezes during large copies
  - **B) Background thread with channel** — responsive UI; show progress bar; cancel support
  - **C) Use `tokio` async runtime** — overkill unless you plan network filesystem support later

### 3.4 Delete

- [ ] `d` then confirmation to delete selected entries
- **Trash vs. permanent:**
  - **A) Permanent delete (`fs::remove_file` / `fs::remove_dir_all`)** — simple, dangerous
  - **B) Trash via `trash` crate** — cross-platform freedesktop/macOS/Windows trash; user expectation on desktop
  - **C) Offer both** — `d` = trash, `D` (shift) = permanent with stronger confirmation
- Recommendation: start with option C

### 3.5 Rename

- [ ] `r` opens inline rename editor (pre-filled with current name)
- [ ] `Esc` to cancel, `Enter` to confirm
- [ ] Validate: no `/`, no empty name, warn on overwrite

### 3.6 Create directory / file

- [ ] `m` (mkdir) to create a new directory — opens text input for name
- [ ] `n` (new) to create an empty file — opens text input for name
- [ ] Navigate into the newly created directory/select the new file after creation

---

## Phase 4 — Dual Pane and Tabs

### 4.1 Dual-pane mode

- [ ] `Tab` to switch focus between left and right pane (each is an independent directory view)
- [ ] Copy/move targets the opposite pane by default
- [ ] Single-pane mode remains available (toggle with a keybinding)
- **Layout choice:**
  - **A) Side-by-side (orthodox style, like Midnight Commander)** — classic, proven
  - **B) Configurable split direction** — horizontal or vertical

### 4.2 Tabs

- [ ] `Ctrl+t` to open a new tab (new directory context)
- [ ] `Ctrl+w` to close current tab
- [ ] Tab bar rendered at the top with tab names (directory basename)
- [ ] Number keys or `gt`/`gT` (vim-style) to switch tabs

---

## Phase 5 — Configuration and Theming

### 5.1 Configuration file

- [ ] Load config from `$XDG_CONFIG_HOME/rota/config.toml` (or `~/.config/rota/config.toml`)
- [ ] Configurable options: default sort, show hidden, confirm before delete, editor command, pager command
- **Crate choice for TOML parsing:** `toml` (standard) **or** `serde` + `toml` together

### 5.2 Custom keybindings

- [ ] Define key → action mappings in config
- [ ] Built-in default keymap; user overrides specific bindings
- [ ] Print current keymap with `?` (help overlay)

### 5.3 Theming and colors

- [ ] Color scheme defined in config (or a separate `theme.toml`)
- [ ] File-type-based coloring (directories = blue, executables = green, symlinks = cyan, etc.)
- [ ] At minimum support: 16-color ANSI, 256-color, and truecolor
- **Crate choice:** ratatui's built-in `Color` and `Style` are sufficient; no extra crate needed

### 5.4 File-type icons (optional)

- [ ] Nerd Font icon prefixes for known extensions (e.g.  for Rust,  for directories)
- [ ] Graceful fallback to ASCII prefixes (`[DIR]`, `[LNK]`) when Nerd Fonts are not detected
- [ ] Disable via config flag

---

## Phase 6 — Advanced Features

### 6.1 Bookmarks

- [ ] `b` to add/remove current directory from bookmarks
- [ ] `B` (or backtick) to open bookmark list; select to jump
- [ ] Persist bookmarks in config directory

### 6.2 Bulk rename

- [ ] Open selected filenames in `$EDITOR` as a text list; save to apply renames
- [ ] Dry-run preview before executing
- [ ] Inspired by `vidir` / `bulkrename`

### 6.3 Shell integration

- [ ] `!` or `:` to open a command prompt; run arbitrary shell commands in the current directory
- [ ] `o` to open the selected file with `$EDITOR` (or `xdg-open` / `open` on macOS)
- [ ] `cd`-on-exit: optionally print `cwd` on quit so the parent shell can `cd` to it

### 6.4 Undo / operation log

- [ ] Keep a log of all file operations performed in the session
- [ ] `u` to undo the last operation (reverse copy = delete copy; reverse move = move back; reverse delete = restore from trash)
- [ ] Show operation history in a popup

### 6.5 Archive handling

- [ ] Enter `.zip`, `.tar.gz`, `.tar.bz2` etc. as if they were directories (read-only browsing)
- [ ] Extract selected archive to current directory
- [ ] Compress selected files into an archive
- **Crate choice:** `zip` crate for zip; `flate2` + `tar` for tarballs; **or** shell out to system `tar`/`unzip`

### 6.6 Fuzzy finder integration

- [ ] Built-in fuzzy file finder (`Ctrl+f`) that searches recursively from cwd
- [ ] **Alternative:** pipe to external `fzf` if available
- **Crate choice for built-in:** `nucleo` (fast, used by Helix editor) **or** `fuzzy-matcher` **or** `skim`

### 6.7 Git status integration

- [ ] Show per-file git status indicators (modified, staged, untracked, ignored)
- [ ] Subtle color or symbol in the file list
- **Crate choice:** `git2` (libgit2 bindings, full-featured) **or** shell out to `git status --porcelain`
- Trade-off: `git2` adds a heavy native dependency; shelling out is simpler but slower for large repos

---

## Phase 7 — Cross-Platform and Packaging

### 7.1 Cross-platform testing

- [ ] Linux (primary target — TedOS)
- [ ] macOS compatibility
- [ ] Windows compatibility (crossterm already supports it; test terminal behavior, path separators, trash)

### 7.2 Packaging

- [ ] Publish to crates.io
- [ ] Provide pre-built binaries via GitHub Releases (use `cross` or `cargo-dist`)
- [ ] AUR package, Homebrew formula, Nix flake (as community interest warrants)
- [ ] `man` page generated from structured help text

### 7.3 CI / CD

- [ ] GitHub Actions: `cargo clippy`, `cargo test`, `cargo fmt --check` on every PR
- [ ] Integration tests: spawn the TUI in a pseudo-terminal, send keystrokes, assert output
- [ ] Release workflow: tag → build matrix → upload artifacts

---

## Phase 8 — Stretch Goals

These are ideas that go beyond a "standard" TUI file manager. Nice to have, not required for v1.

- [ ] **Image preview** in terminals that support Sixel or Kitty graphics protocol
- [ ] **Split terminal** — embed a shell pane below/beside the file manager
- [ ] **Plugin / extension system** — Lua or WASM-based hooks for custom commands
- [ ] **Remote filesystem browsing** — SFTP/SSH via `russh` or `ssh2` crate
- [ ] **Disk usage visualization** — treemap or bar chart of directory sizes
- [ ] **Mouse support** — crossterm supports mouse events; click to select, scroll wheel, drag to select range
- [ ] **Clipboard integration** — copy file paths or file contents to system clipboard (`arboard` crate)

---

## Dependency Summary

| Need | Option A | Option B | Option C |
|---|---|---|---|
| CLI args | `clap` (derive) | `lexopt` (minimal) | — |
| Time formatting | `chrono` | `time` | manual |
| Trash | `trash` crate | permanent only | — |
| TOML config | `toml` + `serde` | — | — |
| Fuzzy search | `nucleo` | `skim` | external `fzf` |
| Git status | `git2` (libgit2) | shell out to `git` | — |
| Archives | `zip` + `tar` + `flate2` | shell out | — |
| Async operations | `std::thread` + channels | `tokio` | — |

---

## Guiding Principles

1. **Errors are UI, not crashes.** Surface problems in the status bar; never panic on filesystem weirdness.
2. **Destructive ops need confirmation.** No silent deletes, overwrites, or moves.
3. **State → Render → Input.** Keep the architecture clean; all UI is a pure function of `App` state.
4. **Keyboard-first.** Every action must be reachable without a mouse. Mouse support is additive.
5. **Minimal dependencies.** Prefer the standard library and small crates. Every dependency is a maintenance cost.
6. **Progressive complexity.** Each phase builds on the last. The app should be useful at every stage.
