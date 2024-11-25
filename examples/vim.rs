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
    Replace(bool), // Bool for "once only?"
    Visual,
    Operator(char),
}

impl Mode {
    fn block<'a>(&self) -> Block<'a> {
        let help = match self {
            Self::Normal => "type q to quit, type i to enter insert mode",
            Self::Replace(_) | Self::Insert => "type Esc to back to normal mode",
            Self::Visual => "type y to yank, type d to delete, type Esc to back to normal mode",
            Self::Operator(_) => "move cursor to apply operator",
        };
        let title = format!("{} MODE ({})", self, help);
        Block::default().borders(Borders::ALL).title(title)
    }

    fn cursor_style(&self) -> Style {
        let color = match self {
            Self::Normal => Color::Reset,
            Self::Insert => Color::LightBlue,
            Self::Visual => Color::LightYellow,
            Self::Replace(_) => Color::LightRed,
            Self::Operator(_) => Color::LightGreen,
        };
        Style::default().fg(color).add_modifier(Modifier::REVERSED)
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Normal => write!(f, "NORMAL"),
            Self::Insert => write!(f, "INSERT"),
            Self::Visual => write!(f, "VISUAL"),
            Self::Replace(_) => write!(f, "REPLACE"),
            Self::Operator(c) => write!(f, "OPERATOR({})", c),
        }
    }
}

// How the Vim emulation state transitions
enum Transition {
    Nop,
    Mode(Mode),
    Pending(Input),
    Quit,
}

// State of Vim emulation
struct Vim {
    mode: Mode,
    pending: Input, // Pending input to handle a sequence with two keys like gg
}

impl Vim {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            pending: Input::default(),
        }
    }

    fn with_pending(self, pending: Input) -> Self {
        Self {
            mode: self.mode,
            pending,
        }
    }

    // True if the textarea cursor is at the end of its given line
    fn is_before_line_end(textarea: &TextArea<'_>) -> bool {
        let (cursor_line, cursor_char) = textarea.cursor();
        let lines = textarea.lines();
        let line = &lines[cursor_line];
        let line_length = line.chars().count(); // FIXME: Not acceptable-- O(N)

        cursor_char < line_length
    }

    fn transition(&self, input: Input, textarea: &mut TextArea<'_>) -> Transition {
        if input.key == Key::Null {
            return Transition::Nop;
        }

        match self.mode {
            Mode::Normal | Mode::Visual | Mode::Operator(_) => {
                match input {
                    Input {
                        key: Key::Char('h'),
                        ..
                    } |
                    Input {
                        key: Key::Left,
                        ..
                    } => textarea.move_cursor(CursorMove::Back),

                    Input {
                        key: Key::Char('j'),
                        ..
                    } |
                    Input {
                        key: Key::Down,
                        ..
                    } => textarea.move_cursor(CursorMove::Down),

                    Input {
                        key: Key::Char('k'),
                        ..
                    } |
                    Input {
                        key: Key::Up,
                        ..
                    } => textarea.move_cursor(CursorMove::Up),

                    Input {
                        key: Key::Char('l'),
                        ..
                    } |
                    Input {
                        key: Key::Right,
                        ..
                    } => textarea.move_cursor(CursorMove::Forward),

                    Input {
                        key: Key::Char('w'),
                        ..
                    } => textarea.move_cursor(CursorMove::WordForward),
                    Input {
                        key: Key::Char('e'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::WordEnd);
                        if matches!(self.mode, Mode::Operator(_)) {
                            textarea.move_cursor(CursorMove::Forward); // Include the text under the cursor
                        }
                    }
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
                    Input { // Note: Not sorted with j
                        key: Key::Char('J'),
                        ..
                    } => {
                        let mut cursor = textarea.cursor();
                        let mut line_count = 1;

                        if let Some(((from_line, from_idx), (to_line, _))) = textarea.selection_range() {
                            // J with a selection joins all lines selected
                            // If only one line is selected, it acts like normal J,
                            // except on failure the cursor moves to selection start.
                            line_count = (to_line-from_line).max(1);
                            cursor = (from_line, from_idx);
                            textarea.cancel_selection(); // fixme restore
                            textarea.move_cursor(CursorMove::Jump(from_line as u16, from_idx as u16));
                        }

                        for _ in 0..line_count {
                            textarea.move_cursor(CursorMove::End);
                            let success = textarea.delete_line_by_end();
                            if success { // A line existed
                                textarea.insert_char(' ');
                            } else { // In regular vim, joining on the final line is a noop
                                let (c1, c2) = cursor;
                                textarea.move_cursor(CursorMove::Jump(c1 as u16, c2 as u16));
                                // TODO: beep
                            }
                        }
                    }
                    Input {
                        key: Key::Char('D'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('C'),
                        ..
                    } => {
                        textarea.delete_line_by_end();
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('p'),
                        ..
                    } => {
                        textarea.paste();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('u'),
                        ctrl: false,
                        ..
                    } => {
                        textarea.undo();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('r'),
                        ctrl: true,
                        ..
                    } => {
                        textarea.redo();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('x'),
                        ..
                    } => {
                        // FIXME: This check shouldn't be necessary, but the vim example is able to cursor over a terminating newline currently, which real vim can't in normal mode
                        // FIXME: Repeatedly mashing x at the end of a line should delete the entire line right to left
                        if Vim::is_before_line_end(&textarea) {
                            textarea.delete_next_char();
                        }
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('i'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('a'),
                        ..
                    } => {
                        textarea.cancel_selection();

                        if Vim::is_before_line_end(&textarea) {
                            textarea.move_cursor(CursorMove::Forward);
                        }
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('A'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('S'),
                        ..
                    } => {
                        let mut line_count = 1;

                        if let Some(((from_line, from_idx), (to_line, _))) = textarea.selection_range() {
                            // S with a selection clears all lines selected
                            line_count = (to_line-from_line).max(1);

                            textarea.cancel_selection(); // fixme restore
                            textarea.move_cursor(CursorMove::Jump(from_line as u16, from_idx as u16));
                        }

                        textarea.move_cursor(CursorMove::Head);
                        for line_idx in 0..line_count {
                            let (cursor_line, _) = textarea.cursor();
                            let lines = textarea.lines();
                            let line = &lines[cursor_line];

                            if line.len() > 0 {
                                // delete_line_by_end has a special behavior where if you are at the end,
                                // it joins the line with the next. Prevent accidentally triggering this on an empty line.
                                textarea.delete_line_by_end();
                            }

                            if line_idx < line_count-1 {
                                // We are now guaranteed at the end of the line.
                                // Join to next line.
                                textarea.delete_line_by_end();
                            }
                        }

                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('o'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::End);
                        textarea.insert_newline();
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('O'),
                        ..
                    } => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.insert_newline();
                        textarea.move_cursor(CursorMove::Up);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('I'),
                        ..
                    } => {
                        textarea.cancel_selection();
                        textarea.move_cursor(CursorMove::Head);
                        return Transition::Mode(Mode::Insert);
                    }
                    Input {
                        key: Key::Char('r'),
                        ..
                    } => {
                        textarea.cancel_selection(); // FIXME: WRONG!! r when there is a selection replaces the selected text.
                        return Transition::Mode(Mode::Replace(true));
                    }
                    Input {
                        key: Key::Char('R'),
                        ..
                    } => {
                        textarea.cancel_selection(); // FIXME: WRONG!! R when there is a selection acts the same as S (?)
                        return Transition::Mode(Mode::Replace(false));
                    }

                    Input {
                        key: Key::Char('q'),
                        ..
                    } => return Transition::Quit,
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
                    Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Visual);
                    }
                    Input {
                        key: Key::Char('V'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        textarea.move_cursor(CursorMove::End);
                        return Transition::Mode(Mode::Visual);
                    }
                    Input { key: Key::Esc, .. }
                    | Input {
                        key: Key::Char('['),
                        ctrl: true,
                        ..
                    }
                    | Input {
                        key: Key::Char('v'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.cancel_selection();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('g'),
                        ctrl: false,
                        ..
                    } if matches!(
                        self.pending,
                        Input {
                            key: Key::Char('g'),
                            ctrl: false,
                            ..
                        }
                    ) =>
                    {
                        textarea.move_cursor(CursorMove::Top)
                    }
                    Input {
                        key: Key::Char('G'),
                        ctrl: false,
                        ..
                    } => textarea.move_cursor(CursorMove::Bottom),
                    Input {
                        key: Key::Char(c),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Operator(c) => {
                        // Handle yy, dd, cc. (This is not strictly the same behavior as Vim)
                        textarea.move_cursor(CursorMove::Head);
                        textarea.start_selection();
                        let cursor = textarea.cursor();
                        textarea.move_cursor(CursorMove::Down);
                        if cursor == textarea.cursor() {
                            textarea.move_cursor(CursorMove::End); // At the last line, move to end of the line instead
                        }
                    }
                    Input {
                        key: Key::Char(op @ ('y' | 'd' | 'c')),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Normal => {
                        textarea.start_selection();
                        return Transition::Mode(Mode::Operator(op));
                    }
                    Input {
                        key: Key::Char('y'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.copy();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('d'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.cut();
                        return Transition::Mode(Mode::Normal);
                    }
                    Input {
                        key: Key::Char('c'),
                        ctrl: false,
                        ..
                    } if self.mode == Mode::Visual => {
                        textarea.move_cursor(CursorMove::Forward); // Vim's text selection is inclusive
                        textarea.cut();
                        return Transition::Mode(Mode::Insert);
                    }
                    input => return Transition::Pending(input),
                }

                // Handle the pending operator
                match self.mode {
                    Mode::Operator('y') => {
                        textarea.copy();
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('d') => {
                        textarea.cut();
                        Transition::Mode(Mode::Normal)
                    }
                    Mode::Operator('c') => {
                        textarea.cut();
                        Transition::Mode(Mode::Insert)
                    }
                    _ => Transition::Nop,
                }
            }
            Mode::Insert => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('['),
                    ctrl: true,
                    ..
                }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => Transition::Mode(Mode::Normal),
                input => {
                    textarea.input(input); // Use default key mappings in insert mode
                    Transition::Mode(Mode::Insert)
                }
            },
            Mode::Replace(once) => match input {
                Input { key: Key::Esc, .. }
                | Input {
                    key: Key::Char('['),
                    ctrl: true,
                    ..
                }
                | Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => Transition::Mode(Mode::Normal),
                Input { key, .. }  => {
                    if match key { Key::Down | Key::Up | Key::Left | Key::Right => false, _ => true }
                    && Vim::is_before_line_end(&textarea) {
                        textarea.delete_next_char(); // FIXME: Will eat newlines and join into next line, should act like insert at end of line 
                    }
                    textarea.input(input); // Use default key mappings in insert mode
                    if once {
                        Transition::Mode(Mode::Normal)   
                    } else {
                        Transition::Mode(Mode::Replace(false))
                    }
                }
            }
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

    textarea.set_block(Mode::Normal.block());
    textarea.set_cursor_style(Mode::Normal.cursor_style());
    let mut vim = Vim::new(Mode::Normal);

    loop {
        term.draw(|f| f.render_widget(&textarea, f.area()))?;

        vim = match vim.transition(crossterm::event::read()?.into(), &mut textarea) {
            Transition::Mode(mode) if vim.mode != mode => {
                textarea.set_block(mode.block());
                textarea.set_cursor_style(mode.cursor_style());
                Vim::new(mode)
            }
            Transition::Nop | Transition::Mode(_) => vim,
            Transition::Pending(input) => vim.with_pending(input),
            Transition::Quit => break,
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
