# `src/main.rs` — Detailed Walkthrough (Rota File Manager, Phase 0)

This program is a **read-only TUI directory lister**. It opens in a terminal “full-screen” mode and shows:

- **Left panel:** a list of entries (directories first, then files), with a highlighted selection cursor  
- **Right panel:** details about the currently selected entry (name/type/size/path)  
- **Header:** current working directory path  
- **Footer:** key bindings or the most recent error message  

It is intentionally **read-only**: no copy/move/delete yet. The goal is to establish a stable foundation: terminal lifecycle, render loop, input handling, state management, and filesystem reading.

---

## High-level architecture

The code follows a classic TUI pattern:

1. **State**: `App` holds current directory, entries, selection index, and last error.
2. **Render**: `terminal.draw(...)` renders the UI from the current state.
3. **Input**: `crossterm::event` reads key events.
4. **Update**: keys mutate state (move selection / enter dir / go up / refresh).
5. Repeat until quit.

This separation is what makes it easy to extend later without turning into spaghetti.

---

## Imports and dependencies

### Standard library

```rust
use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};
```

- `fs`: used for `read_dir` and metadata.
- `io`: used for terminal I/O and `io::Result`.
- `Path`, `PathBuf`: `PathBuf` is stored in state; `Path` is borrowed where needed.
- `SystemTime`: stores modification timestamps when available.

### Crossterm (terminal + input)

```rust
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
```

- **Raw mode**: keypresses are delivered immediately, no line buffering, no echo.
- **Alternate screen**: draws on a separate buffer, restoring your terminal content on exit.
- **Events**: reads keyboard input.
- `KeyEventKind::Press`: avoids duplicate triggers on some terminals (press/release/repeat differences).

### Ratatui (UI widgets)

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
```

Key pieces:
- `Terminal`: does frame rendering via `draw`.
- `Layout`/`Constraint`: splits the screen.
- Widgets: `List`, `Paragraph`, `Block`.
- `ListState`: tracks highlighted row.
- `Style`/`Modifier`: used to reverse colors on the selected row.

---

## Data model: `Entry`

```rust
#[derive(Clone, Debug)]
struct Entry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<SystemTime>,
}
```

Each `Entry` corresponds to one row in the list.

- `name`: displayed label in the list.
- `path`: used for navigation and details display.
- `is_dir`: used for sorting and directory labeling.
- `size`: only set for files; directories get `None`.
- `modified`: only set when metadata is readable.

Using `Option<T>` is deliberate: filesystem metadata is not guaranteed (permissions, broken links, transient errors).

---

## Application state: `App`

```rust
struct App {
    cwd: PathBuf,
    entries: Vec<Entry>,
    selected: usize,
    last_error: Option<String>,
}
```

- `cwd`: current directory being browsed.
- `entries`: directory listing.
- `selected`: index of highlighted entry.
- `last_error`: any error message that should be shown in the footer.

Design choice: errors become UI text instead of panics, so the TUI stays alive.

---

## `App::new()`

```rust
fn new() -> io::Result<Self> {
    let cwd = std::env::current_dir()?;
    let mut app = Self { ... };
    app.refresh();
    Ok(app)
}
```

- Starts in the process’ current working directory.
- Calls `refresh()` immediately to populate the first listing.

`refresh()` is intentionally not `Result`-returning; it stores errors into `last_error`.

---

## `App::refresh()`

Purpose: rebuild `entries` from the filesystem at `cwd`.

### What it does, step-by-step

1. Clear previous error: `self.last_error = None;`
2. Attempt `fs::read_dir(&self.cwd)`
   - If it fails: set `last_error`, clear `entries`, reset `selected`, return.
3. Iterate entries:
   - If an individual entry fails to read: store error and continue.
4. For each entry:
   - Extract `path`, `name`
   - Read metadata (if possible):
     - `is_dir`
     - `size` (files only)
     - `modified`
5. Sort the listing:
   - directories first
   - then case-insensitive name compare
6. Replace `self.entries`
7. Clamp selection so it remains valid.

### Sorting rule

```rust
entries.sort_by(|a, b| {
    match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    }
});
```

Effect:
- Keeps directories grouped at the top.
- Deterministic ordering for quick navigation.

---

## Selection safety

### `clamp_selection()`

Keeps `selected` in bounds after a refresh:

- If `entries` is empty: `selected = 0`.
- If `selected >= entries.len()`: move it to the last valid index.

### `move_selection(delta: i32)`

Moves selection by a signed amount:

- If empty: do nothing.
- Else: compute next index and clamp within `[0, len-1]`.

This prevents underflow/overflow and eliminates index panics.

---

## Navigation logic

### `enter_selected_dir()`

- Reads the selected entry.
- If it is a directory:
  - `cwd = entry.path`
  - reset selection
  - refresh listing

### `go_parent()`

- Uses `cwd.parent()` (if any).
- Sets `cwd` to parent
- resets selection and refreshes

---

## `selected_entry()`

```rust
fn selected_entry(&self) -> Option<&Entry> {
    self.entries.get(self.selected)
}
```

Safe lookup to drive the details panel.

---

## `human_size(bytes: u64)`

Converts a byte count to a readable string:

- `GiB`, `MiB`, `KiB`, or `B`.

It’s deliberately minimal (no localization, no fancy formatting) because this is Phase 0.

---

## Terminal lifecycle: `main()`

`main()` has exactly one job: **setup → run → teardown**.

### Setup

- `enable_raw_mode()`: immediate keypresses, no echo.
- `EnterAlternateScreen`: full-screen drawing without trashing the normal terminal scrollback.
- Construct `Terminal` with `CrosstermBackend`.
- Clear screen.

### Run

- Calls `run_app(&mut terminal)`

### Teardown

Even if the app loop errors, teardown runs:

- `disable_raw_mode()`
- `LeaveAlternateScreen`
- show cursor again

This prevents the classic “terminal is broken after crash” problem.

---

## The application loop: `run_app(...)`

The loop repeats:

1. Sync `ListState` selection to `app.selected`.
2. Draw UI frame.
3. Poll input (50ms).
4. On key press, mutate `app`.

### Why poll?

`event::poll(Duration::from_millis(50))` keeps CPU usage low but still responsive.

### Input handling

Only `KeyEventKind::Press` triggers actions.

Key mapping:

- `q` → quit
- `j` / `Down` → move down
- `k` / `Up` → move up
- `Enter` → enter directory
- `Backspace` → go to parent directory
- `r` → refresh current directory listing

---

## UI rendering (inside `terminal.draw`)

Ratatui rendering happens inside a closure that receives `Frame`.

### Outer split: left vs right

- Left (60%): navigation/list/status
- Right (40%): details panel

### Left split: header, list, footer

- Header: fixed height 3
- List: takes remaining space
- Footer: fixed height 3

#### Header

A `Paragraph` with a bordered `Block`. The block title is the current path:

- Title: `app.cwd.display().to_string()`
- Body: “Rota File Manager Phase 0 (read-only)”

#### List

Builds `Vec<ListItem>` from `app.entries`:

- Directory entries prefixed with `[DIR] `
- Files prefixed with spaces

Uses `render_stateful_widget` so the selected row is highlighted:

- `highlight_style` uses `Modifier::REVERSED`

#### Footer / Status

If `app.last_error` is set, footer shows:

- `ERROR: ...`

Otherwise it shows key bindings. This is your “operator HUD”.

### Right panel: Details

The details `Paragraph` shows information about the currently selected entry:

- Name
- Type
- Size (pretty formatted for files)
- Modified (currently placeholder)
- Full path

The “modified time” is left as a placeholder so you can choose later whether to bring in a time formatting crate (`time` / `chrono`) or keep it raw.

---

## Overall function of the program

This is a **read-only terminal file browser**:

- Reads the current directory from the filesystem
- Displays a navigable list
- Lets you enter directories and go back to parent
- Shows basic metadata for the selected item
- Never modifies the filesystem

In other words: it’s the smallest real “file manager core” that proves your terminal UI + filesystem plumbing is correct.

---

## What this foundation enables next

Because the code is structured cleanly (state → render → input):

- Add `--path` argument (start anywhere)
- Add filter/search within current directory
- Add preview pane for text files
- Add better metadata formatting
- Add file operations later **behind confirmations** if you choose

Phase 0 already gives you the important thing: **a stable interactive shell that reads directories and navigates**.
