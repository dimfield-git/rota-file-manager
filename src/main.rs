use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};

/// One filesystem entry in the current directory.
/// Keep it minimal now; you can extend it later (permissions, owner, etc).
#[derive(Clone, Debug)]
struct Entry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: Option<u64>,          // only for files (None for dirs or if metadata fails)
    modified: Option<SystemTime>, // None if unknown/unreadable
}

/// The entire TUI application state.
/// This is the core idea: state is pure data; UI renders it; input mutates it.
struct App {
    cwd: PathBuf,
    entries: Vec<Entry>,
    selected: usize,
    last_error: Option<String>, // surface errors in UI instead of crashing
}

impl App {
    /// Create a new app starting in the current working directory.
    fn new() -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let mut app = Self {
            cwd,
            entries: vec![],
            selected: 0,
            last_error: None,
        };
        app.refresh(); // populate entries (errors go to last_error)
        Ok(app)
    }

    /// Re-read directory listing for `cwd`.
    /// Errors are stored in `last_error` but we keep the app running.
    fn refresh(&mut self) {
        self.last_error = None;

        let mut entries: Vec<Entry> = Vec::new();

        let read = fs::read_dir(&self.cwd);
        let iter = match read {
            Ok(i) => i,
            Err(e) => {
                self.last_error = Some(format!("read_dir failed: {e}"));
                self.entries.clear();
                self.selected = 0;
                return;
            }
        };

        for item in iter {
            let item = match item {
                Ok(v) => v,
                Err(e) => {
                    // A single bad entry should not kill the whole refresh.
                    self.last_error = Some(format!("read_dir entry error: {e}"));
                    continue;
                }
            };

            let path = item.path();
            let name = item.file_name().to_string_lossy().to_string();

            // Metadata can fail (permissions, broken links, etc). That's fine.
            let md = item.metadata().ok();
            let is_dir = md.as_ref().map(|m| m.is_dir()).unwrap_or(false);

            let size = md
                .as_ref()
                .and_then(|m| if m.is_file() { Some(m.len()) } else { None });

            let modified = md.as_ref().and_then(|m| m.modified().ok());

            entries.push(Entry {
                name,
                path,
                is_dir,
                size,
                modified,
            });
        }

        // Sort:
        // 1) directories first
        // 2) then by case-insensitive name
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.entries = entries;
        self.clamp_selection();
    }

    /// Make sure `selected` is always within bounds.
    fn clamp_selection(&mut self) {
        if self.entries.is_empty() {
            self.selected = 0;
            return;
        }
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
    }

    /// Move selection by delta (+1 down, -1 up).
    fn move_selection(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }

        let len = self.entries.len() as i32;
        let cur = self.selected as i32;

        let next = (cur + delta).clamp(0, len - 1);
        self.selected = next as usize;
    }

    /// Enter the selected entry if it is a directory.
    fn enter_selected_dir(&mut self) {
        let Some(ent) = self.entries.get(self.selected).cloned() else {
            return;
        };

        if ent.is_dir {
            self.cwd = ent.path;
            self.selected = 0;
            self.refresh();
        }
    }

    /// Move up to parent directory if possible.
    fn go_parent(&mut self) {
        if let Some(parent) = self.cwd.parent().map(Path::to_path_buf) {
            self.cwd = parent;
            self.selected = 0;
            self.refresh();
        }
    }

    /// Get selected entry (if any).
    fn selected_entry(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }
}

/// Convert bytes to a rough human readable size.
/// (Deliberately minimal; you can refine formatting later.)
fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GiB", b / GB)
    } else if b >= MB {
        format!("{:.1} MiB", b / MB)
    } else if b >= KB {
        format!("{:.1} KiB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

/// Main entry point.
/// Responsibilities:
/// - setup terminal (raw mode + alternate screen)
/// - run event loop (draw → input → update state)
/// - restore terminal on exit (even if something goes wrong)
fn main() -> io::Result<()> {
    // --- Terminal setup ---
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Ensure we always restore terminal state.
    let result = run_app(&mut terminal);

    // --- Terminal teardown ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Forward any error from the app loop.
    result
}

/// The interactive loop. Keeps running until user quits.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new()?;
    let mut list_state = ListState::default();

    loop {
        // Keep the UI selection state in sync with app.selected
        if app.entries.is_empty() {
            list_state.select(None);
        } else {
            list_state.select(Some(app.selected));
        }

        // --- Draw ---
        terminal.draw(|f| {
            // Layout: left list + right details panel
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(f.area());

            // Left side further split into header + list + footer
            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // header
                    Constraint::Min(0),    // list
                    Constraint::Length(3), // footer/status
                ])
                .split(chunks[0]);

            // --- Header ---
            let header = Paragraph::new(Line::from(vec![
                Span::raw("Rota File Manager  "),
                Span::styled("Phase 0 (read-only)", Style::default().add_modifier(Modifier::BOLD)),
            ]))
            .block(Block::default().borders(Borders::ALL).title(app.cwd.display().to_string()));
            f.render_widget(header, left[0]);

            // --- List items ---
            let items: Vec<ListItem> = app
                .entries
                .iter()
                .map(|e| {
                    // Keep it ASCII-clean for now.
                    let prefix = if e.is_dir { "[DIR] " } else { "      " };
                    ListItem::new(Line::from(format!("{prefix}{}", e.name)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Entries"))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            f.render_stateful_widget(list, left[1], &mut list_state);

            // --- Footer / status ---
            let help = if let Some(err) = &app.last_error {
                format!("ERROR: {err}")
            } else {
                "Keys: j/k or ↑/↓ move | Enter open dir | Backspace up | r refresh | q quit".to_string()
            };

            let footer = Paragraph::new(help)
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .wrap(Wrap { trim: true });
            f.render_widget(footer, left[2]);

            // --- Right side: details panel ---
            let detail_text = match app.selected_entry() {
                None => "No entries".to_string(),
                Some(e) => {
                    let kind = if e.is_dir { "Directory" } else { "File" };
                    let size = e.size.map(human_size).unwrap_or_else(|| "-".to_string());
                    let modified = match e.modified {
                        None => "-".to_string(),
                        Some(_t) => "known (format later)".to_string(), // keep minimal now
                    };

                    format!(
                        "Name: {}\nType: {}\nSize: {}\nModified: {}\n\nPath:\n{}",
                        e.name,
                        kind,
                        size,
                        modified,
                        e.path.display()
                    )
                }
            };

            let details = Paragraph::new(detail_text)
                .block(Block::default().borders(Borders::ALL).title("Details"))
                .wrap(Wrap { trim: false });

            f.render_widget(details, chunks[1]);
        })?;

        // --- Input ---
        // Poll keeps the UI responsive without busy-looping.
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // KeyEventKind::Press avoids double-trigger on some terminals.
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,

                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),

                    KeyCode::Enter => app.enter_selected_dir(),
                    KeyCode::Backspace => app.go_parent(),

                    KeyCode::Char('r') => app.refresh(),

                    _ => {}
                }
            }
        }
    }

    Ok(())
}

