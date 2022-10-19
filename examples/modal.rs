use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Terminal;
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

enum Mode {
    Normal,
    Insert,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
        }
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = if let Some(path) = env::args().nth(1) {
        let file = fs::File::open(path)?;
        io::BufReader::new(file)
            .lines()
            .collect::<io::Result<_>>()?
    } else {
        TextArea::default()
    };

    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Modal Example"),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref());
    let mut mode = Mode::Normal;
    loop {
        term.draw(|f| {
            let chunks = layout.split(f.size());

            f.render_widget(textarea.widget(), chunks[0]);

            let (row, col) = textarea.cursor();
            let help = if let Mode::Normal = mode {
                " type q for quit, type i to enter insert mode"
            } else {
                " type Esc to leave insert mode"
            };
            let status = Paragraph::new(Spans::from(vec![
                Span::styled(
                    format!(" {} {},{} ", mode, row, col),
                    Style::default().add_modifier(Modifier::REVERSED),
                ),
                Span::raw(help),
            ]));
            f.render_widget(status, chunks[1]);
        })?;

        let input = crossterm::event::read()?.into();
        match mode {
            Mode::Normal => match input {
                // Mappings in normal mode
                Input {
                    key: Key::Char('h'),
                    ..
                } => textarea.move_cursor(CursorMove::Back),
                Input {
                    key: Key::Char('j'),
                    ..
                } => textarea.move_cursor(CursorMove::Down),
                Input {
                    key: Key::Char('k'),
                    ..
                } => textarea.move_cursor(CursorMove::Up),
                Input {
                    key: Key::Char('l'),
                    ..
                } => textarea.move_cursor(CursorMove::Forward),
                Input {
                    key: Key::Char('w'),
                    ..
                } => textarea.move_cursor(CursorMove::WordForward),
                Input {
                    key: Key::Char('b'),
                    ctrl: false,
                    ..
                } => textarea.move_cursor(CursorMove::WordBack),
                Input {
                    key: Key::Char('^'),
                    ..
                } => textarea.move_cursor(CursorMove::Head),
                Input {
                    key: Key::Char('$'),
                    ..
                } => textarea.move_cursor(CursorMove::End),
                Input {
                    key: Key::Char('D'),
                    ..
                } => {
                    textarea.delete_line_by_end();
                }
                Input {
                    key: Key::Char('C'),
                    ..
                } => {
                    textarea.delete_line_by_end();
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('p'),
                    ..
                } => {
                    textarea.paste();
                }
                Input {
                    key: Key::Char('u'),
                    ctrl: false,
                    ..
                } => {
                    textarea.undo();
                }
                Input {
                    key: Key::Char('r'),
                    ctrl: true,
                    ..
                } => {
                    textarea.redo();
                }
                Input {
                    key: Key::Char('x'),
                    ..
                } => {
                    textarea.delete_next_char();
                }
                Input {
                    key: Key::Char('i'),
                    ..
                } => mode = Mode::Insert,
                Input {
                    key: Key::Char('a'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Forward);
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('A'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::End);
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('o'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::End);
                    textarea.insert_newline();
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('O'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Head);
                    textarea.insert_newline();
                    textarea.move_cursor(CursorMove::Up);
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('I'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Head);
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('q'),
                    ..
                } => break,
                Input {
                    key: Key::Char('e'),
                    ctrl: true,
                    ..
                } => textarea.scroll((1, 0)),
                Input {
                    key: Key::Char('y'),
                    ctrl: true,
                    ..
                } => textarea.scroll((-1, 0)),
                Input {
                    key: Key::Char('d'),
                    ctrl: true,
                    ..
                } => textarea.scroll(Scrolling::HalfPageDown),
                Input {
                    key: Key::Char('u'),
                    ctrl: true,
                    ..
                } => textarea.scroll(Scrolling::HalfPageUp),
                Input {
                    key: Key::Char('f'),
                    ctrl: true,
                    ..
                } => textarea.scroll(Scrolling::PageDown),
                Input {
                    key: Key::Char('b'),
                    ctrl: true,
                    ..
                } => textarea.scroll(Scrolling::PageUp),
                _ => {}
            },
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    mode = Mode::Normal; // Back to normal mode with Esc or Ctrl+C
                }
                input => {
                    textarea.input(input); // Use default key mappings in insert mode
                }
            },
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());

    Ok(())
}
