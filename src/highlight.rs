use crate::ratatui::style::Style;
use crate::ratatui::text::Span;
use crate::util::{num_digits, spaces};
#[cfg(feature = "ratatui")]
use ratatui::text::Line;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::iter;
#[cfg(feature = "tuirs")]
use tui::text::Spans as Line;
use unicode_width::UnicodeWidthChar as _;

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

fn display_text(
    s: &str,
    tab_len: u8,
    mut width: usize,
    mask: Option<char>,
) -> (Cow<'_, str>, usize) {
    if let Some(ch) = mask {
        // No tab character processing in the mask case
        let chars = s.chars().count();
        let masked = iter::repeat(ch).take(chars).collect();
        return (Cow::Owned(masked), width - ch.width().unwrap_or(0) * chars);
    }

    let tab = spaces(tab_len);
    let mut buf = String::new();
    for (i, c) in s.char_indices() {
        if c == '\t' {
            if buf.is_empty() {
                buf.reserve(s.len());
                buf.push_str(&s[..i]);
            }
            if tab_len > 0 {
                let len = tab_len as usize - (width % tab_len as usize);
                buf.push_str(&tab[..len]);
                width += len;
            }
        } else {
            if !buf.is_empty() {
                buf.push(c);
            }
            width += c.width().unwrap_or(0);
        }
    }

    if !buf.is_empty() {
        (Cow::Owned(buf), width)
    } else {
        (Cow::Borrowed(s), width)
    }
}

pub struct LineHighlighter<'a> {
    line: &'a str,
    spans: Vec<Span<'a>>,
    boundaries: Vec<(Boundary, usize)>, // TODO: Consider smallvec
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    tab_len: u8,
    mask: Option<char>,
}

impl<'a> LineHighlighter<'a> {
    pub fn new(line: &'a str, cursor_style: Style, tab_len: u8, mask: Option<char>) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
            mask,
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

    pub fn into_spans(self) -> Line<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
            mask,
        } = self;

        if boundaries.is_empty() {
            let (text, _) = display_text(line, tab_len, 0, mask);
            spans.push(Span::styled(text, style_begin));
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            }
            return Line::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });

        let mut boundaries = boundaries.into_iter();
        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];
        let mut width = 0;

        loop {
            if let Some((next_boundary, end)) = boundaries.next() {
                if start < end {
                    let (text, w) = display_text(&line[start..end], tab_len, width, mask);
                    spans.push(Span::styled(text, style));
                    width = w;
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
                    let (text, _) = display_text(&line[start..], tab_len, width, mask);
                    spans.push(Span::styled(text, style));
                }
                if cursor_at_end {
                    spans.push(Span::styled(" ", cursor_style));
                }
                return Line::from(spans);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_line_display_text() {
        assert_eq!(display_text(      "", 0, 0,      None), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(      "", 4, 0,      None), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(      "", 8, 0,      None), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(      "", 0, 0, Some('x')), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(      "", 4, 0, Some('x')), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(      "", 8, 0, Some('x')), (Cow::Borrowed(                 ""), 0));
        assert_eq!(display_text(     "a", 0, 0,      None), (Cow::Borrowed(                "a"), 0));
        assert_eq!(display_text(     "a", 4, 0,      None), (Cow::Borrowed(                "a"), 0));
        assert_eq!(display_text(     "a", 8, 0,      None), (Cow::Borrowed(                "a"), 0));
        assert_eq!(display_text(     "a", 0, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(     "a", 4, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(     "a", 8, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(   "a\t", 0, 0,      None), (Cow::Borrowed(                "a"), 0));
        assert_eq!(display_text(   "a\t", 4, 0,      None), (Cow::Borrowed(             "a   "), 0));
        assert_eq!(display_text(   "a\t", 8, 0,      None), (Cow::Borrowed(         "a       "), 0));
        assert_eq!(display_text(   "a\t", 0, 0, Some('x')), (Cow::Borrowed(               "xx"), 0));
        assert_eq!(display_text(   "a\t", 4, 0, Some('x')), (Cow::Borrowed(               "xx"), 0));
        assert_eq!(display_text(   "a\t", 8, 0, Some('x')), (Cow::Borrowed(               "xx"), 0));
        assert_eq!(display_text(    "\t", 0, 0,      None), (Cow::Borrowed(               "\t"), 0));
        assert_eq!(display_text(    "\t", 4, 0,      None), (Cow::Borrowed(             "    "), 0));
        assert_eq!(display_text(    "\t", 8, 0,      None), (Cow::Borrowed(         "        "), 0));
        assert_eq!(display_text(    "\t", 0, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(    "\t", 4, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(    "\t", 8, 0, Some('x')), (Cow::Borrowed(                "x"), 0));
        assert_eq!(display_text(  "a\tb", 0, 0,      None), (Cow::Borrowed(               "ab"), 0));
        assert_eq!(display_text(  "a\tb", 4, 0,      None), (Cow::Borrowed(            "a   b"), 0));
        assert_eq!(display_text(  "a\tb", 8, 0,      None), (Cow::Borrowed(        "a       b"), 0));
        assert_eq!(display_text(  "a\tb", 0, 0, Some('x')), (Cow::Borrowed(              "xxx"), 0));
        assert_eq!(display_text(  "a\tb", 4, 0, Some('x')), (Cow::Borrowed(              "xxx"), 0));
        assert_eq!(display_text(  "a\tb", 8, 0, Some('x')), (Cow::Borrowed(              "xxx"), 0));
        assert_eq!(display_text("a\t\tb", 0, 0,      None), (Cow::Borrowed(               "ab"), 0));
        assert_eq!(display_text("a\t\tb", 4, 0,      None), (Cow::Borrowed(        "a       b"), 0));
        assert_eq!(display_text("a\t\tb", 8, 0,      None), (Cow::Borrowed("a               b"), 0));
        assert_eq!(display_text("a\t\tb", 0, 0, Some('x')), (Cow::Borrowed(             "xxxx"), 0));
        assert_eq!(display_text("a\t\tb", 4, 0, Some('x')), (Cow::Borrowed(             "xxxx"), 0));
        assert_eq!(display_text("a\t\tb", 8, 0, Some('x')), (Cow::Borrowed(             "xxxx"), 0));
        assert_eq!(display_text("ab\t\t", 0, 0,      None), (Cow::Borrowed(               "ab"), 0));
        assert_eq!(display_text("ab\t\t", 4, 0,      None), (Cow::Borrowed(         "ab      "), 0));
        assert_eq!(display_text("ab\t\t", 8, 0,      None), (Cow::Borrowed( "ab              "), 0));
        assert_eq!(display_text("abcd\t", 4, 0,      None), (Cow::Borrowed(         "abcd    "), 0));
        assert_eq!(display_text(  "あ\t", 0, 0,      None), (Cow::Borrowed(               "あ"), 0));
        assert_eq!(display_text(  "あ\t", 4, 0,      None), (Cow::Borrowed(             "あ  "), 0));
        assert_eq!(display_text(  "🐶\t", 4, 0,      None), (Cow::Borrowed(             "🐶  "), 0));
        assert_eq!(display_text(  "あ\t", 4, 0, Some('x')), (Cow::Borrowed(               "xx"), 0));
    }
}
