use crate::tui::style::Style;
#[cfg(any(
    feature = "ratatui-crossterm",
    feature = "ratatui-termion",
    feature = "ratatui-your-backend",
))]
use crate::tui::text::Line as Spans;

use crate::tui::text::Span;
#[cfg(not(any(
    feature = "ratatui-crossterm",
    feature = "ratatui-termion",
    feature = "ratatui-your-backend",
)))]
use crate::tui::text::Spans;

use crate::util::{num_digits, spaces};
use std::borrow::Cow;
use std::cmp::Ordering;

enum Boundary {
    Cursor(Style),
    #[cfg(feature = "search")]
    Search(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 2,
                #[cfg(feature = "search")]
                Boundary::Search(_) => 1,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }

    fn style(&self) -> Option<Style> {
        match self {
            Boundary::Cursor(s) => Some(*s),
            #[cfg(feature = "search")]
            Boundary::Search(s) => Some(*s),
            Boundary::End => None,
        }
    }
}

fn prepare_line(s: &str, tab_len: u8, mask: Option<char>) -> Cow<'_, str> {
    // two tasks performed here
    // - mask out chars if mask is Some
    // - replace hard tabs by the correct number of spaces
    //   (soft tabs were done earlier in 'insert_tab')
    match mask {
        Some(ch) => {
            // no tab processing in the mask case
            return Cow::Owned(s.chars().map(|_| ch).collect());
        }
        None => {
            let tab = spaces(tab_len);
            let mut buf = String::new();
            let mut col: usize = 0;
            for (i, c) in s.char_indices() {
                if buf.is_empty() {
                    if c == '\t' {
                        buf.reserve(s.len());
                        buf.push_str(&s[..i]);
                        if tab_len > 0 {
                            let len = tab_len as usize - (col % tab_len as usize);
                            buf.push_str(&tab[..len]);
                            col += len;
                        }
                    } else {
                        col += 1;
                    }
                } else if c == '\t' {
                    if tab_len > 0 {
                        let len = tab_len as usize - (col % tab_len as usize);
                        buf.push_str(&tab[..len]);
                        col += len;
                    }
                } else {
                    buf.push(c);
                    col += 1;
                }
            }
            if !buf.is_empty() {
                return Cow::Owned(buf);
            }
        }
    };
    // drop through in the case of no mask and no tabs
    return Cow::Borrowed(s);
}

pub struct LineHighlighter<'a> {
    line: &'a str,
    spans: Vec<Span<'a>>,
    boundaries: Vec<(Boundary, usize)>, // TODO: Consider smallvec
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    tab_len: u8,
}

impl<'a> LineHighlighter<'a> {
    pub fn new(line: &'a str, cursor_style: Style, tab_len: u8) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
        }
    }

    pub fn line_number(&mut self, row: usize, lnum_len: u8, style: Style) {
        let pad = spaces(lnum_len - num_digits(row + 1) + 1);
        self.spans
            .push(Span::styled(format!("{}{} ", pad, row + 1), style));
    }

    pub fn cursor_line(&mut self, cursor_col: usize, style: Style) {
        if let Some((start, c)) = self.line.char_indices().nth(cursor_col) {
            self.boundaries
                .push((Boundary::Cursor(self.cursor_style), start));
            self.boundaries.push((Boundary::End, start + c.len_utf8()));
        } else {
            self.cursor_at_end = true;
        }
        self.style_begin = style;
    }

    #[cfg(feature = "search")]
    pub fn search(&mut self, matches: impl Iterator<Item = (usize, usize)>, style: Style) {
        for (start, end) in matches {
            if start != end {
                self.boundaries.push((Boundary::Search(style), start));
                self.boundaries.push((Boundary::End, end));
            }
        }
    }

    pub fn into_spans(self, mask: Option<char>) -> Spans<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
        } = self;

        if boundaries.is_empty() {
            spans.push(Span::styled(prepare_line(line, tab_len, mask), style_begin));
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            }
            return Spans::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });

        let mut boundaries = boundaries.into_iter();
        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];

        loop {
            if let Some((next_boundary, end)) = boundaries.next() {
                if start < end {
                    spans.push(Span::styled(
                        prepare_line(&line[start..end], tab_len, mask),
                        style,
                    ));
                }

                style = if let Some(s) = next_boundary.style() {
                    stack.push(style);
                    s
                } else {
                    stack.pop().unwrap_or(style_begin)
                };
                start = end;
            } else {
                if start != line.len() {
                    spans.push(Span::styled(
                        prepare_line(&line[start..], tab_len, mask),
                        style,
                    ));
                }
                if cursor_at_end {
                    spans.push(Span::styled(" ", cursor_style));
                }
                return Spans::from(spans);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_line() {
        // fn prepare_line(s: &str, tab_len: u8, mask: Option<char>) -> Cow<'_, str> {
        assert_eq!(prepare_line("", 0, None), Cow::Borrowed(""));
        assert_eq!(prepare_line("", 4, None), Cow::Borrowed(""));
        assert_eq!(prepare_line("", 8, None), Cow::Borrowed(""));
        assert_eq!(prepare_line("", 0, Some('x')), Cow::Borrowed(""));
        assert_eq!(prepare_line("", 4, Some('x')), Cow::Borrowed(""));
        assert_eq!(prepare_line("", 8, Some('x')), Cow::Borrowed(""));
        assert_eq!(prepare_line("a", 0, None), Cow::Borrowed("a"));
        assert_eq!(prepare_line("a", 4, None), Cow::Borrowed("a"));
        assert_eq!(prepare_line("a", 8, None), Cow::Borrowed("a"));
        assert_eq!(prepare_line("a", 0, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("a", 4, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("a", 8, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("a\t", 0, None), Cow::Borrowed("a"));
        assert_eq!(prepare_line("a\t", 4, None), Cow::Borrowed("a   "));
        assert_eq!(prepare_line("a\t", 8, None), Cow::Borrowed("a       "));
        assert_eq!(prepare_line("a\t", 0, Some('x')), Cow::Borrowed("xx"));
        assert_eq!(prepare_line("a\t", 4, Some('x')), Cow::Borrowed("xx"));
        assert_eq!(prepare_line("a\t", 8, Some('x')), Cow::Borrowed("xx"));
        assert_eq!(prepare_line("\t", 0, None), Cow::Borrowed("\t"));
        assert_eq!(prepare_line("\t", 4, None), Cow::Borrowed("    "));
        assert_eq!(prepare_line("\t", 8, None), Cow::Borrowed("        "));
        assert_eq!(prepare_line("\t", 0, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("\t", 4, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("\t", 8, Some('x')), Cow::Borrowed("x"));
        assert_eq!(prepare_line("a\tb", 0, None), Cow::Borrowed("ab"));
        assert_eq!(prepare_line("a\tb", 4, None), Cow::Borrowed("a   b"));
        assert_eq!(prepare_line("a\tb", 8, None), Cow::Borrowed("a       b"));
        assert_eq!(prepare_line("a\tb", 0, Some('x')), Cow::Borrowed("xxx"));
        assert_eq!(prepare_line("a\tb", 4, Some('x')), Cow::Borrowed("xxx"));
        assert_eq!(prepare_line("a\tb", 8, Some('x')), Cow::Borrowed("xxx"));
        assert_eq!(prepare_line("a\t\tb", 0, None), Cow::Borrowed("ab"));
        assert_eq!(prepare_line("a\t\tb", 4, None), Cow::Borrowed("a       b"));
        assert_eq!(
            prepare_line("a\t\tb", 8, None),
            Cow::Borrowed("a               b")
        );
        assert_eq!(prepare_line("a\t\tb", 0, Some('x')), Cow::Borrowed("xxxx"));
        assert_eq!(prepare_line("a\t\tb", 4, Some('x')), Cow::Borrowed("xxxx"));
        assert_eq!(prepare_line("a\t\tb", 8, Some('x')), Cow::Borrowed("xxxx"));
    }
}
