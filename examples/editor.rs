use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use std::borrow::Cow;
use std::env;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

macro_rules! error {
    ($fmt: expr, $($args:tt),+) => {{
        Err(io::Error::new(io::ErrorKind::Other, format!($fmt, $($args),+)))
    }};
    ($msg: expr) => {{
        Err(io::Error::new(io::ErrorKind::Other, $msg))
    }};
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
                io::BufReader::new(fs::File::open(&path)?)
                    .lines()
                    .collect::<Result<_, _>>()?
            } else {
                return error!("{:?} is not a file", path);
            }
        } else {
            TextArea::default()
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
        })
    }

    fn run(&mut self) -> io::Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            );

        loop {
            self.term.draw(|f| {
                let buffer = &self.buffers[self.current];
                let textarea = &buffer.textarea;
                let chunks = layout.split(f.size());
                let widget = textarea.widget();
                f.render_widget(widget, chunks[0]);

                let modified = if buffer.modified { " [modified]" } else { "" };
                let status = format!(
                    "[{}/{}] {:?}{}",
                    self.current + 1,
                    self.buffers.len(),
                    buffer.path,
                    modified,
                );
                let status =
                    Paragraph::new(status).style(Style::default().add_modifier(Modifier::REVERSED));
                f.render_widget(status, chunks[1]);

                let message = if let Some(message) = self.message.take() {
                    Spans::from(Span::raw(message))
                } else {
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled("^Q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to quit, "),
                        Span::styled("^S", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to save, "),
                        Span::styled("^X", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to switch buffer"),
                    ])
                };
                f.render_widget(Paragraph::new(message), chunks[2]);
            })?;

            match Input::from(crossterm::event::read()?) {
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
                    self.message = Some(format!("Switched to buffer #{}", self.current + 1).into());
                }
                Input {
                    key: Key::Char('s'),
                    ctrl: true,
                    ..
                } => {
                    self.buffers[self.current].save()?;
                    self.message = Some("Saved!".into());
                }
                input => {
                    let buffer = &mut self.buffers[self.current];
                    buffer.modified = buffer.textarea.input(input);
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
