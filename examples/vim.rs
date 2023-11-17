use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::BufRead;
use tui_textarea::{CursorMove, Input, Key, Scrolling, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Visual,
    Operator(char),
}

impl Mode {
    fn help_message(&self) -> &'static str {
        match self {
            Self::Normal => "type q to quit, type i to enter insert mode",
            Self::Insert => "type Esc to back to normal mode",
            Self::Visual => "type y to yank, type d to delete, type Esc to back to normal mode",
            Self::Operator(_) => "move cursor to apply operator",
        }
    }

    fn cursor_color(&self) -> Color {
        match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Operator(_) => Color::LightGreen,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Visual => write!(f, "VISUAL"),
            Self::Operator(c) => write!(f, "OPERATOR({})", c),
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

    let mut mode = Mode::Normal;
    let mut pending = Input::default();

    loop {
        // Show help message and current mode in title of the block
        let title = format!("{} MODE ({})", mode, mode.help_message());
        let block = Block::default().borders(Borders::ALL).title(title);
        textarea.set_block(block);

        // Change the cursor color looking at current mode
        let color = mode.cursor_color();
        let style = Style::default().fg(color).add_modifier(Modifier::REVERSED);
        textarea.set_cursor_style(style);

        term.draw(|f| f.render_widget(textarea.widget(), f.size()))?;

        let input: Input = crossterm::event::read()?.into();
        if input.key == Key::Null {
            continue;
        }

        let operator = if let Mode::Operator(op) = mode {
            textarea.start_selection();
            Some(op)
        } else {
            None
        };

        mode = match mode {
            Mode::Normal | Mode::Visual | Mode::Operator(_) => match input {
                // Mappings in normal mode
                Input {
                    key: Key::Char('h'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Back);
                    mode
                }
                Input {
                    key: Key::Char('j'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Down);
                    mode
                }
                Input {
                    key: Key::Char('k'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Up);
                    mode
                }
                Input {
                    key: Key::Char('l'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Forward);
                    mode
                }
                Input {
                    key: Key::Char('w'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::WordForward);
                    mode
                }
                Input {
                    key: Key::Char('b'),
                    ctrl: false,
                    ..
                } => {
                    textarea.move_cursor(CursorMove::WordBack);
                    mode
                }
                Input {
                    key: Key::Char('^'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Head);
                    mode
                }
                Input {
                    key: Key::Char('$'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::End);
                    mode
                }
                Input {
                    key: Key::Char('D'),
                    ..
                } => {
                    textarea.delete_line_by_end();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('C'),
                    ..
                } => {
                    textarea.delete_line_by_end();
                    textarea.cancel_selection();
                    Mode::Insert
                }
                Input {
                    key: Key::Char('p'),
                    ..
                } => {
                    textarea.paste();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('u'),
                    ctrl: false,
                    ..
                } => {
                    textarea.undo();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('r'),
                    ctrl: true,
                    ..
                } => {
                    textarea.redo();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('x'),
                    ..
                } => {
                    textarea.delete_next_char();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('i'),
                    ..
                } => {
                    textarea.cancel_selection();
                    Mode::Insert
                }
                Input {
                    key: Key::Char('a'),
                    ..
                } => {
                    textarea.cancel_selection();
                    textarea.move_cursor(CursorMove::Forward);
                    Mode::Insert
                }
                Input {
                    key: Key::Char('A'),
                    ..
                } => {
                    textarea.cancel_selection();
                    textarea.move_cursor(CursorMove::End);
                    Mode::Insert
                }
                Input {
                    key: Key::Char('o'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::End);
                    textarea.insert_newline();
                    Mode::Insert
                }
                Input {
                    key: Key::Char('O'),
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Head);
                    textarea.insert_newline();
                    textarea.move_cursor(CursorMove::Up);
                    Mode::Insert
                }
                Input {
                    key: Key::Char('I'),
                    ..
                } => {
                    textarea.cancel_selection();
                    textarea.move_cursor(CursorMove::Head);
                    Mode::Insert
                }
                Input {
                    key: Key::Char('q'),
                    ..
                } => break,
                Input {
                    key: Key::Char('e'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll((1, 0));
                    mode
                }
                Input {
                    key: Key::Char('y'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll((-1, 0));
                    mode
                }
                Input {
                    key: Key::Char('d'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll(Scrolling::HalfPageDown);
                    mode
                }
                Input {
                    key: Key::Char('u'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll(Scrolling::HalfPageUp);
                    mode
                }
                Input {
                    key: Key::Char('f'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll(Scrolling::PageDown);
                    mode
                }
                Input {
                    key: Key::Char('b'),
                    ctrl: true,
                    ..
                } => {
                    textarea.scroll(Scrolling::PageUp);
                    mode
                }
                Input {
                    key: Key::Char('v'),
                    ctrl: false,
                    ..
                } if mode == Mode::Normal => {
                    textarea.start_selection();
                    Mode::Visual
                }
                Input {
                    key: Key::Char('V'),
                    ctrl: false,
                    ..
                } if mode == Mode::Normal => {
                    textarea.move_cursor(CursorMove::Head);
                    textarea.start_selection();
                    textarea.move_cursor(CursorMove::End);
                    Mode::Visual
                }
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('v'),
                    ctrl: false,
                    ..
                } if mode == Mode::Visual => {
                    textarea.cancel_selection();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('g'),
                    ctrl: false,
                    ..
                } if matches!(
                    pending,
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    }
                ) =>
                {
                    textarea.move_cursor(CursorMove::Top);
                    pending = Input::default();
                    mode
                }
                Input {
                    key: Key::Char('G'),
                    ctrl: false,
                    ..
                } => {
                    textarea.move_cursor(CursorMove::Bottom);
                    mode
                }
                Input {
                    key: Key::Char(c),
                    ctrl: false,
                    ..
                } if operator == Some(c) => {
                    // Handle yy, dd, cc. (This is not strictly the same behavior as Vim)
                    textarea.move_cursor(CursorMove::Head);
                    textarea.start_selection();
                    let cursor = textarea.cursor();
                    textarea.move_cursor(CursorMove::Down);
                    if cursor == textarea.cursor() {
                        textarea.move_cursor(CursorMove::End); // At the last line, move to end of the line instead
                    }
                    mode
                }
                Input {
                    key: Key::Char(op @ ('y' | 'd' | 'c')),
                    ctrl: false,
                    ..
                } if mode == Mode::Normal => Mode::Operator(op),
                Input {
                    key: Key::Char('y'),
                    ctrl: false,
                    ..
                } if mode == Mode::Visual => {
                    textarea.copy();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('d'),
                    ctrl: false,
                    ..
                } if mode == Mode::Visual => {
                    textarea.cut();
                    Mode::Normal
                }
                Input {
                    key: Key::Char('c'),
                    ctrl: false,
                    ..
                } if mode == Mode::Visual => {
                    textarea.cut();
                    Mode::Insert
                }
                input => {
                    pending = input;
                    mode
                }
            },
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => {
                    Mode::Normal // Back to normal mode with Esc or Ctrl+C
                }
                input => {
                    textarea.input(input); // Use default key mappings in insert mode
                    mode
                }
            },
        };

        if let Some(op) = operator {
            if mode != Mode::Normal && mode != Mode::Visual {
                mode = match op {
                    'y' => {
                        textarea.copy();
                        Mode::Normal
                    }
                    'd' => {
                        textarea.cut();
                        Mode::Normal
                    }
                    'c' => {
                        textarea.cut();
                        Mode::Insert
                    }
                    _ => mode,
                }
            }
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
