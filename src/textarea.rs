use crate::cursor::CursorMove;
use crate::edit::{Edit, EditKind};
use crate::history::EditHistory;
use crate::input::{Input, Key};
use std::sync::atomic::{AtomicU16, Ordering};
use tui::buffer::Buffer;
use tui::layout::Rect;
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
    cursor_line_style: Style,
    scroll_top: (AtomicU16, AtomicU16),
}

impl<'a> Default for TextArea<'a> {
    fn default() -> Self {
        Self {
            lines: vec!["".to_string()],
            block: None,
            style: Style::default(),
            cursor: (0, 0),
            tab: "    ",
            history: EditHistory::new(50),
            cursor_line_style: Style::default().add_modifier(Modifier::UNDERLINED),
            scroll_top: (AtomicU16::new(0), AtomicU16::new(0)),
        }
    }
}

impl<'a> TextArea<'a> {
    pub fn input(&mut self, input: impl Into<Input>) {
        let input = input.into();
        match input {
            Input {
                key: Key::Char(c),
                ctrl: false,
                alt: false,
            } => self.insert_char(c),
            Input {
                key: Key::Tab,
                ctrl: false,
                alt: false,
            } => self.insert_tab(),
            Input {
                key: Key::Char('h'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Backspace,
                ..
            } => self.delete_char(),
            Input {
                key: Key::Char('m'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Enter, ..
            } => self.insert_newline(),
            Input {
                key: Key::Char('k'),
                ctrl: true,
                alt: false,
            } => self.delete_line_by_end(),
            Input {
                key: Key::Char('j'),
                ctrl: true,
                alt: false,
            } => self.delete_line_by_head(),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Down,
                ctrl: false,
                alt: false,
            } => self.move_cursor(CursorMove::Down),
            Input {
                key: Key::Char('p'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Up,
                ctrl: false,
                alt: false,
            } => self.move_cursor(CursorMove::Up),
            Input {
                key: Key::Char('f'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Right,
                ctrl: false,
                alt: false,
            } => self.move_cursor(CursorMove::Forward),
            Input {
                key: Key::Char('b'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Left,
                ctrl: false,
                alt: false,
            } => self.move_cursor(CursorMove::Back),
            Input {
                key: Key::Char('a'),
                ctrl: true,
                alt: false,
            }
            | Input { key: Key::Home, .. }
            | Input {
                key: Key::Left | Key::Char('b'),
                ctrl: true,
                alt: true,
            } => self.move_cursor(CursorMove::Head),
            Input {
                key: Key::Char('e'),
                ctrl: true,
                alt: false,
            }
            | Input { key: Key::End, .. }
            | Input {
                key: Key::Right | Key::Char('f'),
                ctrl: true,
                alt: true,
            } => self.move_cursor(CursorMove::End),
            Input {
                key: Key::Char('<'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Up | Key::Char('p'),
                ctrl: true,
                alt: true,
            } => self.move_cursor(CursorMove::Top),
            Input {
                key: Key::Char('>'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Down | Key::Char('n'),
                ctrl: true,
                alt: true,
            } => self.move_cursor(CursorMove::Bottom),
            Input {
                key: Key::Char('u'),
                ctrl: true,
                alt: false,
            } => self.undo(),
            Input {
                key: Key::Char('r'),
                ctrl: true,
                alt: false,
            } => self.redo(),
            _ => {}
        }

        // Check invariants
        debug_assert!(!self.lines.is_empty(), "no line after {:?}", input);
        let (r, c) = self.cursor;
        debug_assert!(
            self.lines.len() > r,
            "cursor {:?} exceeds max lines {} after {:?}",
            self.cursor,
            self.lines.len(),
            input,
        );
        debug_assert!(
            self.lines[r].chars().count() >= c,
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
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(i, c);
        self.cursor.1 += 1;
        self.push_history(EditKind::InsertChar(c, i), (row, col));
    }

    pub fn insert_str(&mut self, s: &str) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        debug_assert_eq!(
            line.char_indices().find(|(_, c)| *c == '\n'),
            None,
            "string given to insert_str must not contain newline",
        );
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert_str(i, s);
        self.cursor.1 += s.chars().count();
        self.push_history(EditKind::Insert(s.to_string(), i), (row, col));
    }

    pub fn delete_str(&mut self, col: usize, chars: usize) {
        if chars == 0 {
            return;
        }
        let row = self.cursor.0;
        let line = &mut self.lines[row];
        if let Some((i, _)) = line.char_indices().nth(col) {
            // push/pop ' ' to preserve ' ' at the end of line
            line.pop();
            let bytes = line[i..]
                .char_indices()
                .nth(chars)
                .map(|(i, _)| i)
                .unwrap_or(line[i..].len());
            let removed = line[i..i + bytes].to_string();
            line.replace_range(i..i + bytes, "");
            line.push(' ');
            self.cursor = (row, col);
            self.push_history(EditKind::Remove(removed, i), (row, col));
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
            .unwrap_or(line.len());
        let next_line = line[idx..].to_string();
        line.truncate(idx);
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
                let prev_line_end = prev_line.len();
                self.cursor = (row - 1, prev_line.chars().count());
                prev_line.push_str(&line);
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

    pub fn delete_line_by_end(&mut self) {
        self.delete_str(self.cursor.1, usize::MAX);
    }

    pub fn delete_line_by_head(&mut self) {
        self.delete_str(0, self.cursor.1);
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
                if let Some((i, c)) = l.char_indices().nth(self.cursor.1) {
                    let j = i + c.len_utf8();
                    lines.push(Spans::from(vec![
                        Span::styled(&l[..i], self.cursor_line_style),
                        Span::styled(&l[i..j], Style::default().add_modifier(Modifier::REVERSED)),
                        Span::styled(&l[j..], self.cursor_line_style),
                    ]));
                } else {
                    // When cursor is at the end of line
                    lines.push(Spans::from(vec![
                        Span::styled(l.as_str(), self.cursor_line_style),
                        Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)),
                    ]));
                }
            } else {
                lines.push(Spans::from(l.as_str()));
            }
        }
        let inner = Paragraph::new(Text::from(lines)).style(self.style);
        TextAreaWidget {
            scroll_top: &self.scroll_top,
            cursor: (self.cursor.0 as u16, self.cursor.1 as u16),
            block: self.block.clone(),
            inner,
        }
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

    pub fn set_cursor_line_style(&mut self, style: Style) {
        self.cursor_line_style = style;
    }

    pub fn lines(&'a self) -> &'a [String] {
        &self.lines
    }

    /// 0-base character-wise (row, col) cursor position.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }
}

struct TextAreaWidget<'a> {
    // &mut 'a (u16, u16) is not available since TextAreaWidget instance takes over the ownership of TextArea instance.
    // In the case the TextArea instance cannot be accessed from any other objects since it is mutablly borrowed.
    scroll_top: &'a (AtomicU16, AtomicU16),
    cursor: (u16, u16),
    block: Option<Block<'a>>,
    inner: Paragraph<'a>,
}

impl<'a> Widget for TextAreaWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let inner_area = if let Some(b) = self.block.take() {
            let area = b.inner(area);
            self.inner = self.inner.block(b);
            area
        } else {
            area
        };

        let top_row = self.scroll_top.0.load(Ordering::Relaxed);
        let top_col = self.scroll_top.1.load(Ordering::Relaxed);

        fn next_scroll_top(prev_top: u16, cursor: u16, width: u16) -> u16 {
            if cursor < prev_top {
                cursor
            } else if prev_top + width <= cursor {
                cursor + 1 - width
            } else {
                prev_top
            }
        }

        let row = next_scroll_top(top_row, self.cursor.0, inner_area.height);
        let col = next_scroll_top(top_col, self.cursor.1, inner_area.width);

        let scroll = (row, col);
        if scroll != (0, 0) {
            self.inner = self.inner.scroll(scroll);
        }

        // Store scroll top position for rendering on the next tick
        self.scroll_top.0.store(row, Ordering::Relaxed);
        self.scroll_top.1.store(col, Ordering::Relaxed);

        self.inner.render(area, buf);
    }
}
