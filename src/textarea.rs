use crate::cursor::CursorMove;
use crate::edit::{Edit, EditKind};
use crate::history::EditHistory;
use crate::input::{Input, Key};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Paragraph, Widget};

pub struct TextArea<'a> {
    lines: Vec<String>,
    block: Option<Block<'a>>,
    style: Style,
    cursor: (usize, usize), // 0-base
    tab: &'a str,
    history: EditHistory,
}

impl<'a> Default for TextArea<'a> {
    fn default() -> Self {
        Self {
            lines: vec![" ".to_string()],
            block: None,
            style: Style::default(),
            cursor: (0, 0),
            tab: "    ",
            history: EditHistory::new(50),
        }
    }
}

impl<'a> TextArea<'a> {
    pub fn input(&mut self, input: impl Into<Input>) {
        let input = input.into();
        if input.ctrl {
            match input.key {
                Key::Char('h') => self.delete_char(),
                Key::Char('m') => self.insert_newline(),
                Key::Char('p') => self.move_cursor(CursorMove::Up),
                Key::Char('f') => self.move_cursor(CursorMove::Forward),
                Key::Char('n') => self.move_cursor(CursorMove::Down),
                Key::Char('b') => self.move_cursor(CursorMove::Back),
                Key::Char('a') => self.move_cursor(CursorMove::Head),
                Key::Char('e') => self.move_cursor(CursorMove::End),
                Key::Char('u') => self.undo(),
                Key::Char('r') => self.redo(),
                _ => {}
            }
        } else {
            match input.key {
                Key::Char(c) => self.insert_char(c),
                Key::Backspace => self.delete_char(),
                Key::Tab => self.insert_tab(),
                Key::Enter => self.insert_newline(),
                Key::Up => self.move_cursor(CursorMove::Up),
                Key::Right => self.move_cursor(CursorMove::Forward),
                Key::Down => self.move_cursor(CursorMove::Down),
                Key::Left => self.move_cursor(CursorMove::Back),
                Key::Home => self.move_cursor(CursorMove::Head),
                Key::End => self.move_cursor(CursorMove::End),
                _ => {}
            }
        }

        // Check invariants
        debug_assert!(!self.lines.is_empty(), "no line after {:?}", input);
        for (i, l) in self.lines.iter().enumerate() {
            debug_assert!(
                l.ends_with(' '),
                "line {} does not end with space after {:?}: {:?}",
                i + 1,
                input,
                l,
            );
        }
        let (r, c) = self.cursor;
        debug_assert!(
            self.lines.len() > r,
            "cursor {:?} exceeds max lines {} after {:?}",
            self.cursor,
            self.lines.len(),
            input,
        );
        debug_assert!(
            self.lines[r].chars().count() > c,
            "cursor {:?} exceeds max col {} at line {:?} after {:?}",
            self.cursor,
            self.lines[r].chars().count(),
            self.lines[r],
            input,
        );
    }

    fn push_history(&mut self, kind: EditKind, cursor_before: (usize, usize)) {
        let edit = Edit::new(kind, cursor_before, self.cursor);
        self.history.push(edit);
    }

    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        if let Some((i, _)) = line.char_indices().nth(col) {
            line.insert(i, c);
            self.cursor.1 += 1;
            self.push_history(EditKind::InsertChar(c, i), (row, col));
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        debug_assert_eq!(
            line.char_indices().find(|(_, c)| *c == '\n'),
            None,
            "string given to insert_str must not contain newline",
        );
        if let Some((i, _)) = line.char_indices().nth(col) {
            line.insert_str(i, s);
            self.cursor.1 += s.chars().count();
            self.push_history(EditKind::Insert(s.to_string(), i), (row, col));
        }
    }

    pub fn insert_tab(&mut self) {
        if !self.tab.is_empty() {
            let len = self.tab.len() - self.cursor.1 % self.tab.len();
            self.insert_str(&self.tab[..len]);
        }
    }

    pub fn insert_newline(&mut self) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let idx = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len() - 1);
        let next_line = line[idx..].to_string();
        line.truncate(idx);
        line.push(' ');
        self.lines.insert(row + 1, next_line);
        self.cursor = (row + 1, 0);
        self.push_history(EditKind::InsertNewline(idx), (row, col));
    }

    pub fn delete_char(&mut self) {
        let (row, col) = self.cursor;
        if col == 0 {
            if row > 0 {
                let line = self.lines.remove(row);
                let prev_line = &mut self.lines[row - 1];
                prev_line.pop(); // Remove trailing space
                let prev_line_end = prev_line.len();
                prev_line.push_str(&line);
                self.cursor = (row - 1, prev_line.chars().count() - 1);
                self.push_history(EditKind::DeleteNewline(prev_line_end), (row, col));
            }
            return;
        }

        let line = &mut self.lines[row];
        if let Some((i, c)) = line.char_indices().nth(col - 1) {
            line.remove(i);
            self.cursor.1 -= 1;
            self.push_history(EditKind::DeleteChar(c, i), (row, col));
        }
    }

    pub fn move_cursor(&mut self, m: CursorMove) {
        if let Some(cursor) = m.next_cursor(self.cursor, &self.lines) {
            self.cursor = cursor;
        }
    }

    pub fn undo(&mut self) {
        if let Some(cursor) = self.history.undo(&mut self.lines) {
            self.cursor = cursor;
        }
    }

    pub fn redo(&mut self) {
        if let Some(cursor) = self.history.redo(&mut self.lines) {
            self.cursor = cursor;
        }
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        let mut lines = Vec::with_capacity(self.lines.len());
        for (i, l) in self.lines.iter().enumerate() {
            if i == self.cursor.0 {
                let (i, c) = l
                    .char_indices()
                    .nth(self.cursor.1)
                    .unwrap_or((l.len() - 1, ' '));
                let j = i + c.len_utf8();
                lines.push(Spans::from(vec![
                    Span::from(&l[..i]),
                    Span::styled(&l[i..j], Style::default().add_modifier(Modifier::REVERSED)),
                    Span::from(&l[j..]),
                ]));
            } else {
                lines.push(Spans::from(l.as_str()));
            }
        }
        let mut p = Paragraph::new(Text::from(lines)).style(self.style);
        if let Some(b) = &self.block {
            p = p.block(b.clone());
        }
        p
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn set_block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }

    pub fn remove_block(&mut self) {
        self.block = None;
    }

    pub fn set_tab(&mut self, tab: &'a str) {
        assert!(
            tab.chars().all(|c| c == ' '),
            "tab string must consist of spaces but got {:?}",
            tab,
        );
        self.tab = tab;
    }

    pub fn set_max_histories(&mut self, max: usize) {
        self.history = EditHistory::new(max);
    }

    pub fn lines(&'a self) -> impl Iterator<Item = &'a str> {
        self.lines.iter().map(|l| &l[..l.len() - 1]) // Trim last whitespace
    }

    /// 0-base character-wise (row, col) cursor position.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }
}
