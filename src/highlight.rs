use crate::cursor::ScreenCursor;
use crate::ratatui::style::Style;
use crate::ratatui::text::Span;
use crate::screen_map::LinePtr;
use crate::util::{num_digits, spaces};
#[cfg(feature = "ratatui")]
use ratatui::text::Line;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::iter;
#[cfg(feature = "tuirs")]
use tui::text::Spans as Line;
use unicode_width::UnicodeWidthChar;

#[derive(Debug)]
enum Boundary {
    Cursor(Style),
    #[cfg(feature = "search")]
    Search(Style),
    Select(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 3,
                #[cfg(feature = "search")]
                Boundary::Search(_) => 1,
                Boundary::Select(_) => 2,
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
            Boundary::Select(s) => Some(*s),
        }
    }
}

struct DisplayTextBuilder {
    tab_len: u8,
    width: usize,
    mask: Option<char>,
}

impl DisplayTextBuilder {
    fn new(tab_len: u8, mask: Option<char>) -> Self {
        Self {
            tab_len,
            width: 0,
            mask,
        }
    }

    fn build<'s>(&mut self, s: &'s str) -> Cow<'s, str> {
        if let Some(ch) = self.mask {
            // Note: We don't need to track width on masking text since width of tab character is fixed
            let masked = iter::repeat(ch).take(s.chars().count()).collect();
            return Cow::Owned(masked);
        }

        let tab = spaces(self.tab_len);
        let mut buf = String::new();
        for (i, c) in s.char_indices() {
            if c == '\t' {
                if buf.is_empty() {
                    buf.reserve(s.len());
                    buf.push_str(&s[..i]);
                }
                if self.tab_len > 0 {
                    let len = self.tab_len as usize - (self.width % self.tab_len as usize);
                    buf.push_str(&tab[..len]);
                    self.width += len;
                }
            } else {
                if !buf.is_empty() {
                    buf.push(c);
                }
                self.width += c.width().unwrap_or(0);
            }
        }

        if !buf.is_empty() {
            Cow::Owned(buf)
        } else {
            Cow::Borrowed(s)
        }
    }
}

pub struct LineHighlighter<'a> {
    line: &'a str,
    spans: Vec<Span<'a>>,
    // reminder - the usize is the start of a section, in BYTES, not chars
    boundaries: Vec<(Boundary, usize)>, // TODO: Consider smallvec
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    tab_len: u8,
    mask: Option<char>,
    select_style: Style,
    char_indices: std::str::CharIndices<'a>,
    running_width: usize,
}

impl<'a> LineHighlighter<'a> {
    pub fn new(
        line: &'a str,
        cursor_style: Style,
        tab_len: u8,
        mask: Option<char>,
        select_style: Style,
    ) -> Self {
        Self {
            line,
            spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            tab_len,
            mask,
            select_style,
            char_indices: line.char_indices(),
            running_width: 0,
        }
    }
    fn screen_col_to_byteoffset(&mut self, col: usize) -> usize {
        loop {
            assert!(
                self.running_width <= col,
                "running_width {} col {}",
                self.running_width,
                col
            );
            if let Some((i, c)) = self.char_indices.next() {
                if self.running_width == col {
                    return i;
                }
                self.running_width += UnicodeWidthChar::width(c).unwrap_or(0);
            } else {
                return self.line.len();
            }
        }
    }

    pub(crate) fn line_number(&mut self, row: usize, lnum_len: u8, style: Style, lp: &LinePtr) {
        if lp.lp_num == 0 {
            let pad = spaces(lnum_len - num_digits(row + 1) + 1);
            self.spans
                .push(Span::styled(format!("{}{} ", pad, row + 1), style));
        } else {
            self.spans.push(Span::styled(spaces(lnum_len + 2), style));
        };
    }

    pub fn select(&mut self, start: usize, end: usize, style: Style) {
        self.boundaries.push((Boundary::Select(style), start));
        self.boundaries.push((Boundary::End, end));
    }
    pub(crate) fn cursor_line(&mut self, sc: ScreenCursor, style: Style, _lp: &LinePtr) {
        if let Some(c) = sc.char {
            let boff = self.screen_col_to_byteoffset(sc.col);
            self.boundaries
                .push((Boundary::Cursor(self.cursor_style), boff));
            self.boundaries.push((Boundary::End, boff + c.len_utf8()));
        } else {
            self.cursor_at_end = true;
        }
        self.style_begin = style;
    }

    pub(crate) fn select_line(
        &mut self,
        row: usize,
        line: &str,
        start: ScreenCursor,
        end: ScreenCursor,
        lp: &LinePtr,
    ) {
        // is this row selected?

        // note that the input coordinates here are in char offsets not bytes
        // so a lot of this code is converting from char offsets to byte offsets
        if start.row <= row && end.row >= row {
            let mut indices = line.char_indices();
            let llen = line.len();
            match (start.row == row, end.row == row) {
                (true, true) => {
                    // only line
                    self.select(
                        indices.nth(start.col).unwrap_or((llen, 'x')).0,
                        indices.nth(end.col - start.col).unwrap_or((llen, 'x')).0,
                        self.select_style,
                    );
                }
                (true, false) => {
                    // a line in the start of selection

                    // the +1 extends the highlight one beyond end, to show
                    // that the newline is included
                    // same done for middle rows below

                    // but only on the last segment of wrapped line

                    self.select(
                        indices.nth(start.col).unwrap_or((llen, 'x')).0,
                        llen + if lp.last { 1 } else { 0 },
                        self.select_style,
                    );
                }
                (false, true) => {
                    // last line
                    if end.col > 0 {
                        self.select(
                            0,
                            indices.nth(end.col).unwrap_or((llen, 'x')).0,
                            self.select_style,
                        );
                    }
                }
                (false, false) => {
                    // in the middle of selection
                    self.select(0, llen + if lp.last { 1 } else { 0 }, self.select_style);
                }
            }
        }
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

    pub(crate) fn into_spans(self, lp: &LinePtr) -> Line<'a> {
        let Self {
            line,
            mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_at_end,
            mask,
            select_style,
            char_indices: _,
            running_width: _,
        } = self;
        let mut builder = DisplayTextBuilder::new(tab_len, mask);
        let llen = line.len();
        if boundaries.is_empty() {
            spans.push(Span::styled(builder.build(line), style_begin));
            if cursor_at_end {
                spans.push(Span::styled(" ", cursor_style));
            }
            return Line::from(spans);
        }

        boundaries.sort_unstable_by(|(l, i), (r, j)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });
        let mut dont_add_cursor = false;
        let mut style = style_begin;
        let mut start = 0;
        let mut stack = vec![];

        for (next_boundary, end) in boundaries {
            trace!(
                "boundary {:?} {} {} {} {:?}",
                next_boundary,
                start,
                end,
                llen,
                style
            );
            if start < end {
                // add extra select space at line end to indicate
                // that the \n will be deleted / included
                if end > llen && style == select_style && lp.last {
                    spans.push(Span::styled(builder.build(&line[start..end - 1]), style));
                    spans.push(Span::styled(" ", style));
                    dont_add_cursor = true;
                } else {
                    spans.push(Span::styled(builder.build(&line[start..end]), style));
                }
            }

            style = if let Some(s) = next_boundary.style() {
                stack.push(style);
                s
            } else {
                stack.pop().unwrap_or(style_begin)
            };
            start = end;
        }
        if start < llen {
            spans.push(Span::styled(builder.build(&line[start..]), style));
        }
        if cursor_at_end && !dont_add_cursor {
            spans.push(Span::styled(" ", cursor_style));
        }

        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr as _;

    fn build(text: &'static str, tab: u8, mask: Option<char>) -> Cow<'static, str> {
        DisplayTextBuilder::new(tab, mask).build(text)
    }

    #[track_caller]
    fn build_with_offset(offset: usize, text: &'static str, tab: u8) -> Cow<'static, str> {
        let mut b = DisplayTextBuilder::new(tab, None);
        b.width = offset;
        let built = b.build(text);
        let want = offset + built.as_ref().width();
        assert_eq!(b.width, want, "in={:?}, out={:?}", text, built); // Check post condition
        built
    }

    #[test]
    #[rustfmt::skip]
    fn test_line_display_text() {
        assert_eq!(&build(      "",  0,      None),                  "");
        assert_eq!(&build(      "",  4,      None),                  "");
        assert_eq!(&build(      "",  8,      None),                  "");
        assert_eq!(&build(      "",  0, Some('x')),                  "");
        assert_eq!(&build(      "",  4, Some('x')),                  "");
        assert_eq!(&build(      "",  8, Some('x')),                  "");
        assert_eq!(&build(     "a",  0,      None),                 "a");
        assert_eq!(&build(     "a",  4,      None),                 "a");
        assert_eq!(&build(     "a",  8,      None),                 "a");
        assert_eq!(&build(     "a",  0, Some('x')),                 "x");
        assert_eq!(&build(     "a",  4, Some('x')),                 "x");
        assert_eq!(&build(     "a",  8, Some('x')),                 "x");
        assert_eq!(&build(   "a\t",  0,      None),                 "a");
        assert_eq!(&build(   "a\t",  4,      None),              "a   ");
        assert_eq!(&build(   "a\t",  8,      None),          "a       ");
        assert_eq!(&build(   "a\t",  0, Some('x')),                "xx");
        assert_eq!(&build(   "a\t",  4, Some('x')),                "xx");
        assert_eq!(&build(   "a\t",  8, Some('x')),                "xx");
        assert_eq!(&build(    "\t",  0,      None),                "\t");
        assert_eq!(&build(    "\t",  4,      None),              "    ");
        assert_eq!(&build(    "\t",  8,      None),          "        ");
        assert_eq!(&build(    "\t",  0, Some('x')),                 "x");
        assert_eq!(&build(    "\t",  4, Some('x')),                 "x");
        assert_eq!(&build(    "\t",  8, Some('x')),                 "x");
        assert_eq!(&build(  "a\tb",  0,      None),                "ab");
        assert_eq!(&build(  "a\tb",  4,      None),             "a   b");
        assert_eq!(&build(  "a\tb",  8,      None),         "a       b");
        assert_eq!(&build(  "a\tb",  0, Some('x')),               "xxx");
        assert_eq!(&build(  "a\tb",  4, Some('x')),               "xxx");
        assert_eq!(&build(  "a\tb",  8, Some('x')),               "xxx");
        assert_eq!(&build("a\t\tb",  0,      None),                "ab");
        assert_eq!(&build("a\t\tb",  4,      None),         "a       b");
        assert_eq!(&build("a\t\tb",  8,      None), "a               b");
        assert_eq!(&build("a\t\tb",  0, Some('x')),              "xxxx");
        assert_eq!(&build("a\t\tb",  4, Some('x')),              "xxxx");
        assert_eq!(&build("a\t\tb",  8, Some('x')),              "xxxx");
        assert_eq!(&build("a\tb\tc", 0,      None),               "abc");
        assert_eq!(&build("a\tb\tc", 4,      None),         "a   b   c");
        assert_eq!(&build("a\tb\tc", 8,      None), "a       b       c");
        assert_eq!(&build("a\tb\tc", 0, Some('x')),             "xxxxx");
        assert_eq!(&build("a\tb\tc", 4, Some('x')),             "xxxxx");
        assert_eq!(&build("a\tb\tc", 8, Some('x')),             "xxxxx");
        assert_eq!(&build("ab\t\t",  0,      None),                "ab");
        assert_eq!(&build("ab\t\t",  4,      None),          "ab      ");
        assert_eq!(&build("ab\t\t",  8,      None),  "ab              ");
        assert_eq!(&build("abcd\t",  4,      None),          "abcd    ");
        assert_eq!(&build(  "あ\t",  0,      None),                "あ");
        assert_eq!(&build(  "あ\t",  4,      None),              "あ  ");
        assert_eq!(&build(  "🐶\t",  4,      None),              "🐶  ");
        assert_eq!(&build(  "あ\t",  4, Some('x')),                "xx");

        // When the start position of the text is not start of the line (#43)
        assert_eq!(&build_with_offset(1,         "", 0),           "");
        assert_eq!(&build_with_offset(1,        "a", 0),          "a");
        assert_eq!(&build_with_offset(1,       "あ", 0),         "あ");
        assert_eq!(&build_with_offset(1,       "\t", 4),        "   ");
        assert_eq!(&build_with_offset(1,      "a\t", 4),        "a  ");
        assert_eq!(&build_with_offset(1,     "あ\t", 4),        "あ ");
        assert_eq!(&build_with_offset(2,       "\t", 4),         "  ");
        assert_eq!(&build_with_offset(2,      "a\t", 4),         "a ");
        assert_eq!(&build_with_offset(2,     "あ\t", 4),     "あ    ");
        assert_eq!(&build_with_offset(3,      "a\t", 4),      "a    ");
        assert_eq!(&build_with_offset(4,       "\t", 4),       "    ");
        assert_eq!(&build_with_offset(4,      "a\t", 4),       "a   ");
        assert_eq!(&build_with_offset(4,     "あ\t", 4),       "あ  ");
        assert_eq!(&build_with_offset(5,       "\t", 4),        "   ");
        assert_eq!(&build_with_offset(5,      "a\t", 4),        "a  ");
        assert_eq!(&build_with_offset(5,     "あ\t", 4),        "あ ");
        assert_eq!(&build_with_offset(2,     "\t\t", 4),     "      ");
        assert_eq!(&build_with_offset(2,   "a\ta\t", 4),     "a a   ");
        assert_eq!(&build_with_offset(1, "あ\tあ\t", 4),    "あ あ  ");
        assert_eq!(&build_with_offset(2, "あ\tあ\t", 4), "あ    あ  ");
    }

    // TODO: Add tests for LineHighlighter
}
