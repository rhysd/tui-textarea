#[cfg(feature = "ratatui-crossterm")]
use crossterm_026 as crossterm;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use std::borrow::Cow;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[cfg(feature = "ratatui-crossterm")]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
#[cfg(not(feature = "ratatui-crossterm"))]
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

macro_rules! error {
    ($fmt: expr $(, $args:tt)*) => {{
        Err(io::Error::new(io::ErrorKind::Other, format!($fmt $(, $args)*)))
    }};
}

struct SearchBox<'a> {
    textarea: TextArea<'a>,
    open: bool,
}

impl<'a> Default for SearchBox<'a> {
    fn default() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        Self {
            textarea,
            open: false,
        }
    }
}

impl<'a> SearchBox<'a> {
    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
        // Remove input for next search. Do not recreate `self.textarea` instance to keep undo history so that users can
        // restore previous input easily.
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.delete_line_by_head();
    }

    fn height(&self) -> u16 {
        if self.open {
            3
        } else {
            0
        }
    }

    fn input(&mut self, input: Input) -> Option<&'_ str> {
        match input {
            Input {
                key: Key::Enter, ..
            }
            | Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            } => None, // Disable shortcuts which inserts a newline. See `single_line` example
            input => {
                let modified = self.textarea.input(input);
                modified.then(|| self.textarea.lines()[0].as_str())
            }
        }
    }

    fn set_error(&mut self, err: Option<impl Display>) {
        let b = if let Some(err) = err {
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search: {}", err))
                .style(Style::default().fg(Color::Red))
        } else {
            Block::default().borders(Borders::ALL).title("Search")
        };
        self.textarea.set_block(b);
    }
}

struct Buffer<'a> {
    textarea: TextArea<'a>,
    path: PathBuf,
    modified: bool,
}

impl<'a> Buffer<'a> {
    fn new(path: PathBuf) -> io::Result<Self> {
        let mut textarea = if let Ok(md) = path.metadata() {
            if md.is_file() {
                let mut textarea: TextArea = io::BufReader::new(fs::File::open(&path)?)
                    .lines()
                    .collect::<io::Result<_>>()?;
                if textarea.lines().iter().any(|l| l.starts_with('\t')) {
                    textarea.set_hard_tab_indent(true);
                }
                textarea
            } else {
                return error!("{:?} is not a file", path);
            }
        } else {
            TextArea::default() // File does not exist
        };
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        Ok(Self {
            textarea,
            path,
            modified: false,
        })
    }

    fn save(&mut self) -> io::Result<()> {
        if !self.modified {
            return Ok(());
        }
        let mut f = io::BufWriter::new(fs::File::create(&self.path)?);
        for line in self.textarea.lines() {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }
        self.modified = false;
        Ok(())
    }
}

struct Editor<'a> {
    current: usize,
    buffers: Vec<Buffer<'a>>,
    term: Terminal<CrosstermBackend<io::Stdout>>,
    message: Option<Cow<'static, str>>,
    search: SearchBox<'a>,
}

impl<'a> Editor<'a> {
    fn new<I>(paths: I) -> io::Result<Self>
    where
        I: Iterator,
        I::Item: Into<PathBuf>,
    {
        let buffers = paths
            .map(|p| Buffer::new(p.into()))
            .collect::<io::Result<Vec<_>>>()?;
        if buffers.is_empty() {
            return error!("USAGE: cargo run --example editor FILE1 [FILE2...]");
        }
        let mut stdout = io::stdout();
        if !is_raw_mode_enabled()? {
            enable_raw_mode()?;
            crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        }
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;
        Ok(Self {
            current: 0,
            buffers,
            term,
            message: None,
            search: SearchBox::default(),
        })
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            let search_height = self.search.height();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(search_height),
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                );

            self.term.draw(|f| {
                let chunks = layout.split(f.size());

                if search_height > 0 {
                    f.render_widget(self.search.textarea.widget(), chunks[0]);
                }

                let buffer = &self.buffers[self.current];
                let textarea = &buffer.textarea;
                let widget = textarea.widget();
                f.render_widget(widget, chunks[1]);

                // Render status line
                let modified = if buffer.modified { " [modified]" } else { "" };
                let slot = format!("[{}/{}]", self.current + 1, self.buffers.len());
                let path = format!(" {}{} ", buffer.path.display(), modified);
                let (row, col) = textarea.cursor();
                let cursor = format!("({},{})", row + 1, col + 1);
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(slot.len() as u16),
                            Constraint::Min(1),
                            Constraint::Length(cursor.len() as u16),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[2]);
                let status_style = Style::default().add_modifier(Modifier::REVERSED);
                f.render_widget(Paragraph::new(slot).style(status_style), status_chunks[0]);
                f.render_widget(Paragraph::new(path).style(status_style), status_chunks[1]);
                f.render_widget(Paragraph::new(cursor).style(status_style), status_chunks[2]);

                // Render message at bottom
                let message = if let Some(message) = self.message.take() {
                    Spans::from(Span::raw(message))
                } else if search_height > 0 {
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to jump to first match and close, "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to close, "),
                        Span::styled(
                            "^G or ↓ or ^N",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" to search next, "),
                        Span::styled(
                            "M-G or ↑ or ^P",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" to search previous"),
                    ])
                } else {
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to quit, "),
                        Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to save, "),
                        Span::styled("^G", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to search, "),
                        Span::styled("^X", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to switch buffer"),
                    ])
                };
                f.render_widget(Paragraph::new(message), chunks[3]);
            })?;

            if search_height > 0 {
                let textarea = &mut self.buffers[self.current].textarea;
                match crossterm::event::read()?.into() {
                    Input {
                        key: Key::Char('g' | 'n'),
                        ctrl: true,
                        alt: false,
                    }
                    | Input { key: Key::Down, .. } => {
                        if !textarea.search_forward(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        alt: true,
                    }
                    | Input {
                        key: Key::Char('p'),
                        ctrl: true,
                        alt: false,
                    }
                    | Input { key: Key::Up, .. } => {
                        if !textarea.search_back(false) {
                            self.search.set_error(Some("Pattern not found"));
                        }
                    }
                    Input {
                        key: Key::Enter, ..
                    } => {
                        if !textarea.search_forward(true) {
                            self.message = Some("Pattern not found".into());
                        }
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    Input { key: Key::Esc, .. } => {
                        self.search.close();
                        textarea.set_search_pattern("").unwrap();
                    }
                    input => {
                        if let Some(query) = self.search.input(input) {
                            let maybe_err = textarea.set_search_pattern(query).err();
                            self.search.set_error(maybe_err);
                        }
                    }
                }
            } else {
                match crossterm::event::read()?.into() {
                    Input {
                        key: Key::Char('q'),
                        ctrl: true,
                        ..
                    } => break,
                    Input {
                        key: Key::Char('x'),
                        ctrl: true,
                        ..
                    } => {
                        self.current = (self.current + 1) % self.buffers.len();
                        self.message =
                            Some(format!("Switched to buffer #{}", self.current + 1).into());
                    }
                    Input {
                        key: Key::Char('s'),
                        ctrl: true,
                        ..
                    } => {
                        self.buffers[self.current].save()?;
                        self.message = Some("Saved!".into());
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: true,
                        ..
                    } => {
                        self.search.open();
                    }
                    input => {
                        let buffer = &mut self.buffers[self.current];
                        buffer.modified = buffer.textarea.input(input);
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> Drop for Editor<'a> {
    fn drop(&mut self) {
        self.term.show_cursor().unwrap();
        if !is_raw_mode_enabled().unwrap() {
            return;
        }
        disable_raw_mode().unwrap();
        crossterm::execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
    }
}

fn main() -> io::Result<()> {
    Editor::new(env::args_os().skip(1))?.run()
}
