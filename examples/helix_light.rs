use crossterm::cursor::{SetCursorStyle, Show};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Position;
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use tui_textarea::{CursorMove, Scrolling, TextArea, TextWrapMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
    Replace,
    Select,
    GoTo(bool),
    Space,
}

impl Mode {
    fn block<'a>(&self) -> Block<'a> {
        let help = match self {
            Mode::Normal => "[q] quit | [i] insert mode | [v] select mode",
            Mode::Insert => "[esc] back",
            Mode::Replace => "any char to replace selection",
            Mode::Select => "[esc] back",
            Mode::GoTo(_) => "[g] start of file or go to line | [e] end of file",
            Mode::Space => "[y] clipboard yank | [p] clipboard paste",
        };
        let title = format!("{} MODE ({})", self, help);
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .bg(Color::Rgb(25, 35, 48))
    }

    fn cursor_style(&self) -> Option<SetCursorStyle> {
        match self {
            Mode::Select => Some(SetCursorStyle::SteadyUnderScore),
            Mode::Normal => Some(SetCursorStyle::SteadyBlock),
            Mode::Replace => Some(SetCursorStyle::SteadyUnderScore),
            Mode::Insert => Some(SetCursorStyle::SteadyBar),
            Mode::GoTo(_) => None,
            Mode::Space => None,
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Mode::Normal => write!(f, "NOR"),
            Mode::Insert => write!(f, "INS"),
            Mode::Replace => write!(f, "REP"),
            Mode::Select => write!(f, "SEL"),
            Mode::GoTo(_) => write!(f, "GO"),
            Mode::Space => write!(f, "SPC"),
        }
    }
}

// How the Vim emulation state transitions
enum Transition {
    Nop,
    Quit,
}

// State of Helix (light) emulation
struct Helix {
    mode: Mode,
    selected_number: Option<u16>,
}

impl Helix {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            selected_number: None,
        }
    }

    fn set_select_mode(&mut self, tx: &mut TextArea) {
        self.mode = Mode::Select;
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn set_replace_mode(&mut self, tx: &mut TextArea) {
        self.mode = Mode::Replace;
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn set_normal_mode(&mut self, tx: &mut TextArea) {
        self.mode = Mode::Normal;
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn set_insert_mode(&mut self, tx: &mut TextArea) {
        self.mode = Mode::Insert;
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn set_goto_mode(&mut self, insert: bool, tx: &mut TextArea) {
        self.mode = Mode::GoTo(insert);
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn set_space_mode(&mut self, tx: &mut TextArea) {
        self.mode = Mode::Space;
        self.change_cursor();
        tx.set_block(self.mode.block());
    }

    fn change_cursor(&self) {
        if let Some(style) = self.mode.cursor_style() {
            execute!(std::io::stdout(), Show, style).unwrap();
        }
    }

    pub fn init(&mut self, tx: &mut TextArea) {
        self.set_normal_mode(tx);
    }

    fn handle(&mut self, code: KeyCode, modifiers: KeyModifiers, tx: &mut TextArea) -> Transition {
        let exit = Transition::Quit;
        let skip = Transition::Nop;

        let mut clear_selected_number = true;
        if self.selected_number.is_some() {
            if let KeyCode::Char(c) = code {
                if c.is_ascii_digit() || c.eq(&'g') || c.eq(&'G') || c.eq(&'x') {
                    clear_selected_number = false;
                }
            }
        }
        if clear_selected_number {
            self.selected_number = None;
        }

        // TODO
        // - review logics for n actions eg v3wd to delete 3 words

        match self.mode {
            Mode::Space => {
                match code {
                    KeyCode::Char('p') => {
                        tx.cancel_selection();
                        let mut cp = arboard::Clipboard::new().unwrap();
                        let content = cp.get_text().unwrap();
                        if !content.is_empty() {
                            tx.move_cursor(CursorMove::Forward);
                            tx.insert_str(content);
                            tx.move_cursor(CursorMove::Back);
                        }
                    }
                    KeyCode::Char('P') => {
                        let mut cp = arboard::Clipboard::new().unwrap();
                        let content = cp.get_text().unwrap();
                        if !content.is_empty() {
                            if let Some(((row, col), (_, _))) = tx.selection_range() {
                                tx.move_cursor(CursorMove::Jump(row as u16, col as u16));
                            }
                            tx.cancel_selection();
                            tx.insert_str(content);
                        } else {
                            tx.cancel_selection();
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if !tx.is_selecting() {
                            tx.start_selection();
                        }
                        if tx.cut() {
                            let mut cp = arboard::Clipboard::new().unwrap();
                            cp.set_text(tx.yank_text()).unwrap();
                            tx.paste();
                        }
                        tx.cancel_selection();
                    }
                    KeyCode::Char('R') => {
                        let mut cp = arboard::Clipboard::new().unwrap();
                        let content = cp.get_text().unwrap();
                        if !content.is_empty() {
                            if !tx.is_selecting() {
                                tx.start_selection();
                            }
                            tx.cut();
                            tx.insert_str(content);
                        }
                        tx.cancel_selection();
                    }
                    _ => (),
                };
                self.set_normal_mode(tx);
            }

            Mode::GoTo(insert) => {
                match code {
                    KeyCode::Char('g') if self.selected_number.is_some() => {
                        let row = self.selected_number.unwrap();
                        self.selected_number = None;
                        tx.move_cursor(CursorMove::Jump(row.saturating_sub(1), 0));
                    }
                    KeyCode::Char('g') => {
                        tx.move_cursor(CursorMove::Top);
                        tx.move_cursor(CursorMove::Head);
                    }
                    KeyCode::Char('e') => {
                        tx.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
                    }
                    KeyCode::Char('h') => {
                        tx.move_cursor(CursorMove::Head);
                    }
                    KeyCode::Char('l') => {
                        tx.move_cursor(CursorMove::End);
                    }
                    KeyCode::Char('s') => {
                        tx.move_cursor(CursorMove::HeadNonSpace);
                    }
                    _ => (),
                };
                if insert {
                    self.set_insert_mode(tx);
                } else {
                    self.set_normal_mode(tx);
                }
            }

            Mode::Replace => match code {
                KeyCode::Char(c) => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    let len = tx.yank_text().len();
                    let repl = (0..len).map(|_| c).collect::<String>();
                    tx.insert_str(repl);
                    tx.move_cursor(CursorMove::Back);
                    self.set_normal_mode(tx);
                }
                _ => {
                    self.set_normal_mode(tx);
                }
            },

            Mode::Normal | Mode::Select => match code {
                // EXIT (TEMP)
                KeyCode::Char('q') => return exit,
                // EXIT SELECT MODE
                KeyCode::Esc if self.mode.eq(&Mode::Select) => {
                    tx.cancel_selection();
                    self.set_normal_mode(tx);
                }
                // SPACE
                KeyCode::Char(' ') => {
                    self.set_space_mode(tx);
                }
                // MOVE TO LINE
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if let Some(c) = c.to_digit(10) {
                        let s = self
                            .selected_number
                            .map(|i| i.saturating_mul(10).saturating_add(c as u16))
                            .unwrap_or(c as u16);
                        self.selected_number = Some(s);
                    }
                }
                KeyCode::Char('G') if self.selected_number.is_some() => {
                    let row = self.selected_number.unwrap();
                    self.selected_number = None;
                    tx.move_cursor(CursorMove::Jump(row.saturating_sub(1), 0));
                }
                // MOVEMENTS
                KeyCode::Char('h') | KeyCode::Left => tx.move_cursor(CursorMove::Back),
                KeyCode::Char('j') | KeyCode::Down => tx.move_cursor(CursorMove::Down),
                KeyCode::Char('k') | KeyCode::Up => tx.move_cursor(CursorMove::Up),
                KeyCode::Char('l') | KeyCode::Right => tx.move_cursor(CursorMove::Forward),
                KeyCode::Char('w') => tx.move_cursor(CursorMove::WordForward),
                KeyCode::Char('W') => tx.move_cursor(CursorMove::WordSpacingForward),
                KeyCode::Char('e') => tx.move_cursor(CursorMove::WordEnd),
                KeyCode::Char('E') => tx.move_cursor(CursorMove::WordSpacingEnd),
                KeyCode::Home => tx.move_cursor(CursorMove::Head),
                KeyCode::End => tx.move_cursor(CursorMove::End),
                KeyCode::Char('b') if modifiers.eq(&KeyModifiers::CONTROL) => {
                    tx.scroll(Scrolling::PageUp)
                }
                KeyCode::PageUp => tx.scroll(Scrolling::PageUp),
                KeyCode::Char('b') => tx.move_cursor(CursorMove::WordBack),
                KeyCode::Char('B') => tx.move_cursor(CursorMove::WordSpacingBack),
                KeyCode::Char('f') if modifiers.eq(&KeyModifiers::CONTROL) => {
                    tx.scroll(Scrolling::PageDown)
                }
                KeyCode::PageDown => tx.scroll(Scrolling::PageDown),
                KeyCode::Char('u') if modifiers.eq(&KeyModifiers::CONTROL) => {
                    tx.scroll(Scrolling::HalfPageUp)
                }
                KeyCode::Char('d') if modifiers.eq(&KeyModifiers::CONTROL) => {
                    tx.scroll(Scrolling::HalfPageDown)
                }
                // CHANGES
                KeyCode::Char('r') => {
                    self.set_replace_mode(tx);
                }
                KeyCode::Char('R') => {
                    let new_text = tx.yank_text();
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    tx.insert_str(new_text);
                    tx.move_cursor(CursorMove::Back);
                    self.set_normal_mode(tx);
                }
                KeyCode::Insert | KeyCode::Char('i') => {
                    if let Some(((row, col), (_, _))) = tx.selection_range() {
                        tx.move_cursor(CursorMove::Jump(row as u16, col as u16));
                        tx.cancel_selection();
                    }
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('a') => {
                    tx.cancel_selection();
                    tx.move_cursor(CursorMove::Forward);
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('I') => {
                    tx.cancel_selection();
                    tx.move_cursor(CursorMove::Head);
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('A') => {
                    tx.cancel_selection();
                    tx.move_cursor(CursorMove::End);
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('o') => {
                    // TODO SET FUNCTION
                    let (row, _) = tx.cursor();
                    let line = &tx.lines()[row];
                    let tab_len = tx.tab_length() as usize;
                    let mut tab_count = 0;
                    for (i, c) in line.chars().enumerate() {
                        if c.eq(&' ') && i % tab_len == 3 {
                            tab_count += 1;
                        } else if !c.eq(&' ') {
                            break;
                        }
                    }
                    tx.cancel_selection();
                    tx.move_cursor(CursorMove::End);
                    tx.insert_newline();
                    for _ in 0..tab_count {
                        tx.insert_tab();
                    }
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('O') => {
                    // TODO SET FUNCTION
                    let (row, _) = tx.cursor();
                    let line = &tx.lines()[row];
                    let tab_len = tx.tab_length() as usize;
                    let mut tab_count = 0;
                    for (i, c) in line.chars().enumerate() {
                        if c.eq(&' ') && i % tab_len == 3 {
                            tab_count += 1;
                        } else if !c.eq(&' ') {
                            break;
                        }
                    }
                    tx.cancel_selection();
                    if row == 0 {
                        tx.move_cursor(CursorMove::Head);
                        tx.insert_newline();
                        tx.move_cursor(CursorMove::Up);
                        for _ in 0..tab_count {
                            tx.insert_tab();
                        }
                    } else {
                        tx.move_cursor(CursorMove::Up);
                        tx.move_cursor(CursorMove::End);
                        tx.insert_newline();
                        for _ in 0..tab_count {
                            tx.insert_tab();
                        }
                    }
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('u') => {
                    tx.undo();
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('U') => {
                    tx.redo();
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('y') => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.copy();
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('p') => {
                    tx.cancel_selection();
                    if !tx.yank_text().is_empty() {
                        tx.move_cursor(CursorMove::Forward);
                        tx.insert_str(tx.yank_text());
                        tx.move_cursor(CursorMove::Back);
                    }
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('P') => {
                    if let Some(((row, col), (_, _))) = tx.selection_range() {
                        tx.move_cursor(CursorMove::Jump(row as u16, col as u16));
                    }
                    tx.cancel_selection();
                    tx.insert_str(tx.yank_text());
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('>') if self.mode.eq(&Mode::Normal) => {
                    let (row, col) = tx.cursor();
                    tx.move_cursor(CursorMove::Head);
                    tx.insert_str(tx.indent());
                    tx.move_cursor(CursorMove::Jump(row as u16, col as u16 + 4));
                }
                KeyCode::Char('<') if self.mode.eq(&Mode::Normal) => {
                    let (row, col) = tx.cursor();
                    if tx.lines()[row].starts_with(tx.indent()) {
                        tx.move_cursor(CursorMove::Head);
                        tx.delete_str(tx.indent().len());
                        tx.move_cursor(CursorMove::Jump(
                            row as u16,
                            (col as u16).saturating_sub(4),
                        ));
                    }
                }
                KeyCode::Char('d') if modifiers.eq(&KeyModifiers::ALT) => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    tx.set_yank_text("");
                    self.set_normal_mode(tx);
                }
                KeyCode::Delete | KeyCode::Char('d') => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('c') if modifiers.eq(&KeyModifiers::ALT) => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    tx.set_yank_text("");
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('c') => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    tx.cut();
                    self.set_insert_mode(tx);
                }
                KeyCode::Char('v') if self.mode.eq(&Mode::Normal) => {
                    tx.start_selection();
                    self.set_select_mode(tx);
                }
                KeyCode::Char('v') => {
                    tx.cancel_selection();
                    self.set_normal_mode(tx);
                }
                // SELECTION
                KeyCode::Char('%') => {
                    tx.move_cursor(CursorMove::Top);
                    tx.move_cursor(CursorMove::Head);
                    tx.start_selection();
                    tx.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
                }
                KeyCode::Char('x') if self.selected_number.is_some() => {
                    let s = self.selected_number.unwrap().saturating_sub(1);
                    if !tx.is_selecting() {
                        let (row, _) = tx.cursor();
                        tx.move_cursor(CursorMove::Head);
                        tx.start_selection();
                        tx.move_cursor(CursorMove::Jump(row as u16 + s, u16::MAX));
                        self.set_select_mode(tx);
                    } else if let Some(((r1, _), (r2, _))) = tx.selection_range() {
                        tx.move_cursor(CursorMove::Jump(r1 as u16, 0));
                        tx.start_selection();
                        tx.move_cursor(CursorMove::Jump(r2 as u16 + s, u16::MAX));
                    }
                    self.selected_number = None;
                }
                KeyCode::Char('x') => {
                    if !tx.is_selecting() {
                        tx.move_cursor(CursorMove::Head);
                        tx.start_selection();
                        tx.move_cursor(CursorMove::End);
                        self.set_select_mode(tx);
                    } else if let Some(((r1, c1), (r2, c2))) = tx.selection_range() {
                        let l = &tx.lines()[r2];
                        if c1 == 0 && c2 == l.chars().count() {
                            tx.move_cursor(CursorMove::Down);
                            tx.move_cursor(CursorMove::End);
                        } else {
                            tx.move_cursor(CursorMove::Jump(r1 as u16, 0));
                            tx.start_selection();
                            tx.move_cursor(CursorMove::Jump(r2 as u16, u16::MAX));
                        }
                    }
                }
                // GOTO MODE
                KeyCode::Char('g') => {
                    self.set_goto_mode(self.mode.eq(&Mode::Insert), tx);
                }
                // EDITING
                KeyCode::Char('`') if modifiers.eq(&KeyModifiers::ALT) => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    if tx.cut() {
                        let text = tx.yank_text().to_uppercase();
                        tx.insert_str(text);
                    }
                    tx.move_cursor(CursorMove::Back);
                    self.set_normal_mode(tx);
                }
                KeyCode::Char('`') => {
                    if !tx.is_selecting() {
                        tx.start_selection();
                    }
                    if tx.cut() {
                        let text = tx.yank_text().to_lowercase();
                        tx.insert_str(text);
                    }
                    tx.move_cursor(CursorMove::Back);
                    self.set_normal_mode(tx);
                }
                _ => (),
            },
            Mode::Insert => match code {
                KeyCode::Esc => {
                    self.set_normal_mode(tx);
                }
                KeyCode::Enter => {
                    let (row, _) = tx.cursor();
                    let line = &tx.lines()[row];
                    let tab_len = tx.tab_length() as usize;
                    let mut tab_count = 0;
                    for (i, c) in line.chars().enumerate() {
                        if c.eq(&' ') && i % tab_len == 3 {
                            tab_count += 1;
                        } else if !c.eq(&' ') {
                            break;
                        }
                    }
                    tx.insert_newline();
                    for _ in 0..tab_count {
                        tx.insert_tab();
                    }
                }
                KeyCode::Backspace => {
                    let (row, col) = tx.cursor();
                    let line = &tx.lines()[row];
                    let comp = (0..col).map(|_| ' ').collect::<String>();
                    let to_delete = if line.starts_with(&comp) {
                        // let nb_tabs = col.saturating_sub(1) / tx.tab_length() as usize;
                        let t = tx.tab_length() as usize;
                        if col % t == 0 {
                            t
                        } else {
                            col % t
                        }
                        //
                    } else {
                        1
                    };
                    for _ in 0..to_delete {
                        tx.delete_char();
                    }
                }
                // MOVEMENTS
                KeyCode::Left => tx.move_cursor(CursorMove::Back),
                KeyCode::Down => tx.move_cursor(CursorMove::Down),
                KeyCode::Up => tx.move_cursor(CursorMove::Up),
                KeyCode::Right => tx.move_cursor(CursorMove::Forward),
                KeyCode::Home => tx.move_cursor(CursorMove::Head),
                KeyCode::End => tx.move_cursor(CursorMove::End),
                KeyCode::PageUp => tx.scroll(Scrolling::PageUp),
                KeyCode::PageDown => tx.scroll(Scrolling::PageDown),
                _ => {
                    tx.input_without_shortcuts(KeyEvent::new(code, modifiers));
                }
            },
        }

        skip
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    // let mut stdout = stdout.lock();

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

    textarea.set_style(
        Style::new()
            .bg(Color::Rgb(25, 35, 48))
            .fg(Color::Rgb(205, 206, 207)),
    );
    textarea.set_tab_length(4);
    textarea.set_cursor_style(Style::default().bg(Color::Rgb(41, 57, 79)));
    textarea.set_cursor_line_style(Style::default().bg(Color::Rgb(41, 57, 79)));
    textarea.set_selection_style(Style::default().bg(Color::Rgb(60, 83, 114)));
    textarea.set_cursor_line_fullwidth();
    textarea.set_cursor_hidden();
    textarea.set_selection_inclusive();
    textarea.set_line_number_style(Style::new().fg(Color::Rgb(113, 131, 155)));
    textarea.set_textwrap(TextWrapMode::Word);

    textarea.set_block(Mode::Normal.block());

    let mut helix = Helix::new(Mode::Normal);
    helix.init(&mut textarea);

    loop {
        term.draw(|f| {
            let area = f.area();
            // area.width = 20;
            // area.height = 20;
            f.render_widget(&textarea, area);

            let (y, x) = textarea.screen_cursor();
            f.set_cursor_position(Position {
                x: x as u16 + 1,
                y: y as u16 + 1,
            });
        })?;

        if let crossterm::event::Event::Key(keyevent) = crossterm::event::read()? {
            match helix.handle(keyevent.code, keyevent.modifiers, &mut textarea) {
                Transition::Nop => (),
                Transition::Quit => break,
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
