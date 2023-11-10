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

enum Mode {
    Normal,
    Insert,
}

impl Mode {
    fn help_message(&self) -> &'static str {
        match self {
            Self::Normal => "type q to quit, type i to enter insert mode",
            Self::Insert => "type Esc to back to normal mode",
        }
    }

    fn cursor_color(&self) -> Color {
        match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
        }
    }
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
    textarea.set_selection_style(Style::default().bg(Color::Red));
    let mut mode = Mode::Normal;
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

        // since this sample doesnt use input a lot of the time
        // it must call should_select

        textarea.should_select(&input);
        match mode {
            Mode::Normal => match input {
                // Mappings in normal mode

                // note that the navigation keys can be pressed with or without shift
                // if the user is selecting text
                // so we have to look for 'h' and 'H' etc.
                Input {
                    key: Key::Char('h' | 'H'),
                    ..
                } => textarea.move_cursor(CursorMove::Back),
                Input {
                    key: Key::Char('j' | 'J'),
                    ..
                } => textarea.move_cursor(CursorMove::Down),
                Input {
                    key: Key::Char('k' | 'K'),
                    ..
                } => textarea.move_cursor(CursorMove::Up),
                Input {
                    key: Key::Char('l' | 'L'),
                    ..
                } => textarea.move_cursor(CursorMove::Forward),
                Input {
                    key: Key::Char('w' | 'W'),
                    ..
                } => textarea.move_cursor(CursorMove::WordForward),
                Input {
                    key: Key::Char('b' | 'B'),
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
                    key: Key::Char('y'),
                    ctrl: false,
                    ..
                } => {
                    textarea.copy();
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
                    // stop selection is called to prevent 'A' being interpreted as selecting
                    textarea.stop_selection();
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
                    // stop selection is called to prevent 'O' being interpreted as selecting
                    textarea.stop_selection();
                    textarea.move_cursor(CursorMove::Head);
                    textarea.insert_newline();
                    textarea.move_cursor(CursorMove::Up);
                    mode = Mode::Insert;
                }
                Input {
                    key: Key::Char('I'),
                    ..
                } => {
                    textarea.stop_selection();
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
