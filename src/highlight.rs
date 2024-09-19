use crate::ratatui::style::Style;
use crate::ratatui::text::Span;
use crate::util::{num_digits, spaces, Pos};
use crate::wordwrap::TextWrapMode;
use crate::wordwrap::{compute_slices, get_cursor_col};
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
    Select(Style),
    #[cfg(feature = "search")]
    Search(Style),
    End,
}

impl Boundary {
    fn cmp(&self, other: &Boundary) -> Ordering {
        fn rank(b: &Boundary) -> u8 {
            match b {
                Boundary::Cursor(_) => 3,
                #[cfg(feature = "search")]
                Boundary::Search(_) => 2,
                Boundary::Select(_) => 1,
                Boundary::End => 0,
            }
        }
        rank(self).cmp(&rank(other))
    }

    fn style(&self) -> Option<Style> {
        match self {
            Boundary::Cursor(s) => Some(*s),
            Boundary::Select(s) => Some(*s),
            #[cfg(feature = "search")]
            Boundary::Search(s) => Some(*s),
            Boundary::End => None,
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
    // spans: Vec<Span<'a>>,
    boundaries: Vec<(Boundary, usize, usize)>, // TODO: Consider smallvec
    style_begin: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    cursor_line_fullwidth: bool,
    cursor_col: usize,
    tab_len: u8,
    mask: Option<char>,
    select_at_end: bool,
    select_style: Style,
    current_line: bool,
    current_line_style: Style,
    width: u16,
    cursor_hidden: bool,
    line_number: Option<(Span<'a>, Span<'a>)>,
}

#[allow(clippy::too_many_arguments)]
impl<'a> LineHighlighter<'a> {
    pub fn new(
        line: &'a str,
        cursor_style: Style,
        cursor_line_fullwidth: bool,
        tab_len: u8,
        mask: Option<char>,
        select_style: Style,
        width: u16,
        cursor_hidden: bool,
    ) -> Self {
        Self {
            line,
            // spans: vec![],
            boundaries: vec![],
            style_begin: Style::default(),
            cursor_at_end: false,
            cursor_style,
            cursor_line_fullwidth,
            cursor_col: 0,
            tab_len,
            mask,
            select_at_end: false,
            select_style,
            current_line: false,
            current_line_style: Style::default(),
            width,
            cursor_hidden,
            line_number: None,
        }
    }

    pub fn line_number(&mut self, row: usize, lnum_len: u8, style: Style) {
        let pad = spaces(lnum_len - num_digits(row + 1) + 1);
        let span = Span::styled(format!("{}{} ", pad, row + 1), style);
        let pad = spaces(lnum_len + 1);
        let empty_span = Span::raw(format!("{} ", pad));
        self.line_number = Some((span, empty_span));
    }

    pub fn cursor_line(&mut self, cursor_col: usize, style: Style) {
        self.current_line = true;
        self.current_line_style = style;
        self.cursor_col = cursor_col;
        if let Some((start, c)) = self.line.char_indices().nth(cursor_col) {
            if !self.cursor_hidden {
                self.boundaries
                    .push((Boundary::Cursor(self.cursor_style), start, cursor_col));
                self.boundaries
                    .push((Boundary::End, start + c.len_utf8(), cursor_col + 1));
            }
        } else {
            self.cursor_at_end = true;
        }
        self.style_begin = style;
    }

    #[cfg(feature = "search")]
    pub fn search(
        &mut self,
        matches: impl Iterator<Item = ((usize, usize), (usize, usize))>,
        style: Style,
    ) {
        for ((start_byte, end_byte), (start_char, end_char)) in matches {
            if start_byte != end_byte {
                // TODO for wordwrap
                self.boundaries
                    .push((Boundary::Search(style), start_byte, start_char));
                self.boundaries.push((Boundary::End, end_byte, end_char));
            }
        }
    }

    pub fn selection(&mut self, current_row: usize, start: Pos, end: Pos) {
        let ((start_off, end_off), (start_col, end_col)) = if current_row == start.row {
            if start.row == end.row {
                ((start.offset, end.offset), (start.col, end.col))
            } else {
                self.select_at_end = true;
                (
                    (start.offset, self.line.len()),
                    (start.col, self.line.chars().count()),
                )
            }
        } else if current_row == end.row {
            ((0, end.offset), (0, end.col))
        } else if start.row < current_row && current_row < end.row {
            self.select_at_end = true;
            ((0, self.line.len()), (0, self.line.chars().count()))
        } else {
            return;
        };
        if start_off != end_off {
            self.boundaries
                .push((Boundary::Select(self.select_style), start_off, start_col));
            self.boundaries.push((Boundary::End, end_off, end_col));
        }
    }

    pub fn into_spans(
        self,
        textwrap: &Option<TextWrapMode>,
    ) -> (Vec<Line<'a>>, Option<(usize, usize)>) {
        #[allow(unused_variables)]
        let Self {
            line,
            // mut spans,
            mut boundaries,
            tab_len,
            style_begin,
            cursor_style,
            cursor_line_fullwidth,
            cursor_col,
            cursor_at_end,
            mask,
            select_at_end,
            select_style,
            current_line,
            current_line_style,
            width,
            cursor_hidden,
            line_number,
        } = self;

        if width == 0 {
            return (vec![], current_line.then(|| (0, 0)));
        }

        let slices = match textwrap {
            Some(mode) => compute_slices(line, width as usize, mode),
            // Some(mode) => match mode {
            //     TextWrapMode::Width => compute_slices(line, width as usize),
            //     TextWrapMode::Word => compute_slices_words(line, width as usize, true),
            //     TextWrapMode::WORD => compute_slices_words(line, width as usize, false),
            // },
            None => vec![((0, line.chars().count()), (0, line.len()))],
        };
        // let slices = compute_slices(wordwrap, width as usize, line);
        // let slices = compute_slices_words(textwrap, width as usize, line);

        boundaries.sort_unstable_by(|(l, i, _), (r, j, _)| match i.cmp(j) {
            Ordering::Equal => l.cmp(r),
            o => o,
        });

        // let mut screen_cursor;
        // let mut screen_cursor = (0, 0);
        // let mut cursor_row = 0;

        let mut lines = vec![];
        // let mut last_style = style_begin;
        let mut stack: Vec<Style> = vec![];
        let mut first_line_number = true;
        let mut last_line_number = false;
        for (i, (slice_chars, slice_bytes)) in slices.iter().enumerate() {
            if i == slices.len() - 1 {
                last_line_number = true;
            }
            let line = into_spans_line(
                line,
                *slice_chars,
                *slice_bytes,
                &mut boundaries,
                style_begin,
                // last_style,
                cursor_at_end,
                cursor_style,
                cursor_line_fullwidth,
                tab_len,
                mask,
                select_at_end,
                select_style,
                current_line,
                current_line_style,
                width,
                &mut stack, // wordwrap,
                &line_number,
                first_line_number,
                last_line_number,
            );
            first_line_number = false;
            lines.push(line);
            // last_style = style;
            // if has_cursor {
            //     cursor_row = i;
            // }
            // if let Some(c) = cursor {
            //     // eprintln!("{c}");
            //     screen_cursor = (i, c);
            // }
        }

        // if textwrap.is_none() {
        //     screen_cursor.1 = cursor_col;
        // }
        let screen_cursor = if let Some(mode) = textwrap {
            get_cursor_col(line, cursor_col, width as usize, mode).unwrap_or((0, 0))
        } else {
            (0, cursor_col)
        };

        (lines, current_line.then(|| screen_cursor))
    }
}

// fn compute_slices_words(
//     wordwrap: bool,
//     width: usize,
//     line: &str,
// ) -> Vec<((usize, usize), (usize, usize))> {
//     if wordwrap {
//         // let t = bwrap::Wrapper::new(line, width, )

//         let mut wrapper = bwrap::EasyWrapper::new(line, width).unwrap();
//         let wrapped = wrapper
//             .wrap_use_style(bwrap::WrapStyle::MayBrk(None, None))
//             .unwrap();

//         let wrapped = bwrap::wrap!(line, width);
//         println!("{:?}", wrapped);
//         return vec![];
//         // let wrapped_text = wrapped.lines();

//         // let wrapped_text = textwrap::wrap(line, width);

//         // textwrap::wrap(
//         //     line,
//         //     textwrap::Options::new(width).line_ending(textwrap::LineEnding::LF),
//         // );

//         let mut start_byte = 0;
//         let mut start_char = 0;

//         let lines = wrapped.lines();
//         // let count = lines.count();

//         let mut slices = vec![];
//         for (index, part) in lines.into_iter().enumerate() {
//             // for (index, part) in wrapped_text.iter().enumerate() {
//             let mut end_char = part.chars().count() + start_char;
//             let mut end_byte = part.len() + start_byte;
//             // if index != wrapped_text.len() - 1 {
//             // // if index != wrapped_text.len() - 1 {
//             //     end_char += 1;
//             //     end_byte += 1;
//             // }
//             slices.push(((start_char, end_char), (start_byte, end_byte)));
//             start_byte = end_byte;
//             start_char = end_char;
//         }

//         // println!("{:?}", slices);

//         // let full_lines_count = line.chars().count() / width;

//         // let mut slices = vec![];
//         // for i in 0..full_lines_count {
//         //     let offset = i * width;
//         //     let (first, _) = line.char_indices().skip(offset).take(1).last().unwrap();
//         //     let (last, _) = line
//         //         .char_indices()
//         //         .skip(offset + width)
//         //         .take(1)
//         //         .last()
//         //         .unwrap_or((line.len(), ' '));
//         //     slices.push(((offset, offset + width), (first, last)));
//         // }
//         // if line.is_empty() {
//         //     slices.push(((0, 0), (0, 0)));
//         // } else if line.chars().count() % width != 0 {
//         //     let offset = full_lines_count * width;
//         //     let (first, _) = line.char_indices().skip(offset).take(1).last().unwrap();
//         //     slices.push(((offset, line.chars().count()), (first, line.len())));
//         // } else {
//         //     let c = line.chars().count();
//         //     let l = line.len();
//         //     slices.push(((c, c), (l, l)));
//         // }
//         slices
//     } else {
//         vec![((0, line.chars().count()), (0, line.len()))]
//     }
// }

#[allow(clippy::too_many_arguments)]
fn into_spans_line<'a>(
    line: &'a str,
    slice_chars: (usize, usize),
    slice_bytes: (usize, usize),
    boundaries: &mut [(Boundary, usize, usize)], // TODO: Consider smallvec
    style_begin: Style,
    // last_style: Style,
    cursor_at_end: bool,
    cursor_style: Style,
    cursor_line_fullwidth: bool,
    tab_len: u8,
    mask: Option<char>,
    select_at_end: bool,
    select_style: Style,
    current_line: bool,
    current_line_style: Style,
    width: u16,
    stack: &mut Vec<Style>,
    line_number: &Option<(Span<'a>, Span<'a>)>,
    first_line_number: bool,
    last_line_number: bool,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = vec![];
    let mut builder = DisplayTextBuilder::new(tab_len, mask);

    let cline = if slice_bytes.0 != slice_bytes.1 {
        &line[slice_bytes.0..slice_bytes.1]
    } else {
        ""
    };

    if let Some((span, empty_span)) = line_number {
        if first_line_number {
            spans.push(span.clone());
        } else {
            spans.push(empty_span.clone());
        }
    }

    if boundaries.is_empty() {
        let built = builder.build(cline);
        if !built.is_empty() {
            spans.push(Span::styled(built, style_begin));
        }

        // let mut has_cursor = None;

        // TODO CHECK IF NEEDS TO BE REVIEWED FOR CURSORHIDE ENHANCEMENT
        if last_line_number && cursor_at_end {
            // has_cursor = true;
            // has_cursor = Some(cline.chars().count());
            spans.push(Span::styled(" ", cursor_style));
        } else if select_at_end {
            spans.push(Span::styled(" ", select_style));
        }

        if cursor_line_fullwidth && current_line {
            let len = width.saturating_sub(cline.chars().count() as u16);
            let empty = (0..len).map(|_| " ").collect::<String>();
            spans.push(Span::styled(empty, current_line_style));
        }
        // eprintln!("no boundaries");
        // eprintln!("{:?}", has_cursor);
        return Line::from(spans);
    }

    // let mut has_cursor = None;
    // let mut has_cursor = false;

    // boundaries.sort_unstable_by(|(l, i, _), (r, j, _)| match i.cmp(j) {
    //     Ordering::Equal => l.cmp(r),
    //     o => o,
    // });

    // let mut style = last_style;
    // let mut style = style_begin;
    // let mut style = stack.last().cloned().unwrap_or(style_begin);
    let mut style = stack.pop().unwrap_or(style_begin);
    // spans.push(Span::raw(format!("{:?}", style)));
    let mut start = slice_bytes.0;
    // let mut stack = vec![];

    for (next_boundary, end_byte, end_char) in boundaries {
        if *end_char < slice_chars.0 {
            continue;
        }

        let end = (*end_byte).min(slice_bytes.1);

        if start < end {
            spans.push(Span::styled(builder.build(&line[start..end]), style));
            if style.eq(&cursor_style) {
                // has_cursor = Some(3);
                // has_cursor = Some(*end_char);
                // has_cursor = Some(slice_chars.0 + *end_char);
                // has_cursor = true;
            }
            if let Boundary::Cursor(_) = next_boundary {
                // has_cursor = Some(3);
                // has_cursor = Some(*end_char);
                // has_cursor = Some(slice_chars.0 + *end_char);
                // has_cursor = true;
            }
        }

        start = end;
        if end >= slice_bytes.1 {
            stack.push(style);
            break;
        }
        // if start < *end {
        //     spans.push(Span::styled(builder.build(&line[start..*end]), style));
        // }
        style = if let Some(s) = next_boundary.style() {
            stack.push(style);
            s
        } else {
            stack.pop().unwrap_or(style_begin)
        };
    }

    if start < slice_bytes.1 {
        spans.push(Span::styled(
            builder.build(&line[start..slice_bytes.1]),
            style,
        ));
    }
    // if start != line.len() {
    //     spans.push(Span::styled(builder.build(&line[start..]), style));
    // }

    if last_line_number && cursor_at_end {
        // has_cursor = true;
        // has_cursor = Some(cline.chars().count());
        spans.push(Span::styled(" ", cursor_style));
    } else if select_at_end {
        spans.push(Span::styled(" ", select_style));
    }

    if cursor_line_fullwidth {
        let len = width.saturating_sub(cline.chars().count() as u16);
        let empty = (0..len).map(|_| " ").collect::<String>();
        spans.push(Span::styled(empty, current_line_style));
    }

    Line::from(spans)
    // (Line::from(spans), style)
}

// Tests for spans don't work with tui-rs
#[cfg(all(test, feature = "ratatui"))]
mod tests {
    use super::*;
    use crate::ratatui::style::Color;
    use std::fmt::Debug;
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
    fn line_display_text() {
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

    fn assert_spans<T: Debug>(lh: LineHighlighter, want: &[(&str, Style)], context: T) {
        let line = &lh.into_spans(&None).0[0]; // TODO
        let have = line
            .spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style))
            .collect::<Vec<_>>();
        assert_eq!(&have, want, "Test case: {context:?}");
    }

    const DEFAULT: Style = Style::new();
    const CUR: Style = Style::new().bg(Color::Red); // Cursor
    #[allow(unused)]
    const SEARCH: Style = Style::new().bg(Color::Green);
    const SEL: Style = Style::new().bg(Color::Blue);
    const LINE: Style = Style::new().bg(Color::Gray);
    const LNUM: Style = Style::new().bg(Color::Yellow);

    #[test]
    fn into_spans_normal_line() {
        let tests = [
            ("", &[][..]),
            ("abc", &[("abc", DEFAULT)][..]),
            ("a\tb\tc", &[("a   b   c", DEFAULT)][..]),
        ];
        for test in tests {
            let (line, want) = test;
            let lh = LineHighlighter::new(line, CUR, false, 4, None, SEL, 50, false);
            assert_spans(lh, want, test);
        }
    }

    #[test]
    fn into_spans_cursor_line() {
        let tests = [
            ("", 0, &[(" ", CUR)][..]),
            ("a", 0, &[("a", CUR)][..]),
            ("a", 1, &[("a", LINE), (" ", CUR)][..]),
            ("あいう", 0, &[("あ", CUR), ("いう", LINE)][..]),
            ("あいう", 1, &[("あ", LINE), ("い", CUR), ("う", LINE)][..]),
            ("あいう", 2, &[("あい", LINE), ("う", CUR)][..]),
            ("a\tb", 1, &[("a", LINE), ("   ", CUR), ("b", LINE)][..]),
        ];

        for test in tests {
            let (line, col, want) = test;
            let mut lh = LineHighlighter::new(line, CUR, false, 4, None, SEL, 50, false);
            lh.cursor_line(col, LINE);
            assert_spans(lh, want, test);
        }
    }

    #[test]
    fn into_spans_line_number() {
        let tests = [
            (0, 1, &[(" 1 ", LNUM)][..]),
            (123, 3, &[(" 124 ", LNUM)][..]),
            (123, 5, &[("   124 ", LNUM)][..]),
        ];
        for test in tests {
            let (row, len, want) = test;
            let mut lh = LineHighlighter::new("", CUR, false, 4, None, SEL, 50, false);
            lh.line_number(row, len, LNUM);
            assert_spans(lh, want, test);
        }
    }

    #[cfg(feature = "search")]
    #[test]
    fn into_spans_search() {
        let tests = [
            ("abcde", &[((0, 5), (0, 5))][..], &[("abcde", SEARCH)][..]),
            (
                "abcde",
                &[((0, 1), (0, 1)), ((2, 3), (2, 3)), ((4, 5), (4, 5))][..],
                &[
                    ("a", SEARCH),
                    ("b", DEFAULT),
                    ("c", SEARCH),
                    ("d", DEFAULT),
                    ("e", SEARCH),
                ][..],
            ),
            (
                "abcde",
                &[((1, 2), (1, 2)), ((3, 4), (3, 4))][..],
                &[
                    ("a", DEFAULT),
                    ("b", SEARCH),
                    ("c", DEFAULT),
                    ("d", SEARCH),
                    ("e", DEFAULT),
                ][..],
            ),
            (
                "abcde",
                &[((0, 2), (0, 2)), ((2, 4), (2, 4)), ((4, 5), (4, 5))][..],
                &[("ab", SEARCH), ("cd", SEARCH), ("e", SEARCH)][..],
            ),
            ("abcde", &[((1, 1), (1, 1))][..], &[("abcde", DEFAULT)][..]),
            (
                "あいうえお",
                &[((0, 3), (0, 1)), ((6, 9), (2, 3)), ((12, 15), (4, 5))][..],
                &[
                    ("あ", SEARCH),
                    ("い", DEFAULT),
                    ("う", SEARCH),
                    ("え", DEFAULT),
                    ("お", SEARCH),
                ][..],
            ),
            (
                "\ta\tb\t",
                &[((0, 1), (0, 1)), ((2, 3), (2, 3)), ((3, 4), (3, 4))][..],
                &[
                    ("    ", SEARCH),
                    ("a", DEFAULT),
                    ("   ", SEARCH),
                    ("b", SEARCH),
                    ("   ", DEFAULT),
                ][..],
            ),
        ];

        for test in tests {
            let (line, matches, want) = test;
            let mut lh = LineHighlighter::new(line, CUR, false, 4, None, SEL, 50, false);
            lh.search(matches.iter().copied(), SEARCH);
            assert_spans(lh, want, test);
        }
    }

    #[test]
    fn into_spans_selection() {
        let tests = [
            // (line, (row, start_row, start_off, end_row, end_off), want)
            ("abc", (0, 1, 0, 2, 0), &[("abc", DEFAULT)][..]),
            ("abc", (1, 1, 0, 1, 1), &[("a", SEL), ("bc", DEFAULT)][..]),
            ("abc", (1, 1, 2, 1, 3), &[("ab", DEFAULT), ("c", SEL)][..]),
            ("abc", (1, 1, 0, 1, 3), &[("abc", SEL)][..]),
            ("abc", (1, 1, 0, 2, 0), &[("abc", SEL), (" ", SEL)][..]),
            (
                "abc",
                (1, 1, 2, 2, 0),
                &[("ab", DEFAULT), ("c", SEL), (" ", SEL)][..],
            ),
            ("abc", (1, 1, 3, 2, 0), &[("abc", DEFAULT), (" ", SEL)][..]),
            ("abc", (2, 1, 0, 3, 0), &[("abc", SEL), (" ", SEL)][..]),
            ("abc", (2, 1, 0, 2, 0), &[("abc", DEFAULT)][..]),
            ("abc", (2, 1, 0, 2, 2), &[("ab", SEL), ("c", DEFAULT)][..]),
            ("abc", (2, 1, 0, 2, 3), &[("abc", SEL)][..]),
            (
                "ab\t",
                (1, 1, 2, 2, 0),
                &[("ab", DEFAULT), ("  ", SEL), (" ", SEL)][..],
            ),
            ("a\tb", (2, 1, 0, 3, 0), &[("a   b", SEL), (" ", SEL)][..]),
            (
                "a\tb",
                (2, 1, 0, 2, 2),
                &[("a   ", SEL), ("b", DEFAULT)][..],
            ),
        ];

        for test in tests {
            let (line, (row, start_row, start_off, end_row, end_off), want) = test;
            let mut lh = LineHighlighter::new(line, CUR, false, 4, None, SEL, 50, false);
            // lh.selection(row, start_row, start_off, end_row, end_off);
            lh.selection(
                row,
                Pos {
                    row: start_row,
                    col: start_off,
                    offset: start_off,
                },
                Pos {
                    row: end_row,
                    col: end_off,
                    offset: end_off,
                },
            );
            assert_spans(lh, want, test);
        }
    }

    #[test]
    fn into_spans_mixed_highlights() {
        let tests = [
            (
                "cursor on selection",
                {
                    let mut lh = LineHighlighter::new("abcde", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(2, LINE);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 1,
                            offset: 1,
                        },
                        Pos {
                            row: 0,
                            col: 4,
                            offset: 4,
                        },
                    );
                    // lh.selection(0, 0, 1, 0, 4);
                    lh
                },
                &[("a", LINE), ("b", SEL), ("c", CUR), ("d", SEL), ("e", LINE)][..],
            ),
            #[cfg(feature = "search")]
            (
                "cursor + selection + search",
                {
                    let mut lh =
                        LineHighlighter::new("abcdefg", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(3, LINE);
                    // lh.selection(0, 0, 2, 0, 5);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 2,
                            offset: 2,
                        },
                        Pos {
                            row: 0,
                            col: 5,
                            offset: 5,
                        },
                    );
                    lh.search([((1, 2), (1, 2)), ((5, 6), (5, 6))].into_iter(), SEARCH);
                    lh
                },
                &[
                    ("a", LINE),
                    ("b", SEARCH),
                    ("c", SEL),
                    ("d", CUR),
                    ("e", SEL),
                    ("f", SEARCH),
                    ("g", LINE),
                ][..],
            ),
            (
                "selection + cursor at end",
                {
                    let mut lh = LineHighlighter::new("ab", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(2, LINE);
                    // lh.selection(0, 0, 1, 2, 0);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 1,
                            offset: 1,
                        },
                        Pos {
                            row: 2,
                            col: 0,
                            offset: 0,
                        },
                    );
                    lh
                },
                &[("a", LINE), ("b", SEL), (" ", CUR)][..],
            ),
            (
                "cursor at start of selection",
                {
                    let mut lh = LineHighlighter::new("abcd", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(1, LINE);
                    // lh.selection(0, 0, 1, 0, 3);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 1,
                            offset: 1,
                        },
                        Pos {
                            row: 0,
                            col: 3,
                            offset: 3,
                        },
                    );
                    lh
                },
                &[("a", LINE), ("b", CUR), ("c", SEL), ("d", LINE)][..],
            ),
            (
                "cursor at end of selection",
                {
                    let mut lh = LineHighlighter::new("abcd", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(2, LINE);
                    // lh.selection(0, 0, 1, 0, 3);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 1,
                            offset: 1,
                        },
                        Pos {
                            row: 0,
                            col: 3,
                            offset: 3,
                        },
                    );
                    lh
                },
                &[("a", LINE), ("b", SEL), ("c", CUR), ("d", LINE)][..],
            ),
            (
                "cursor covers selection",
                {
                    let mut lh = LineHighlighter::new("abc", CUR, false, 4, None, SEL, 50, false);
                    lh.cursor_line(1, LINE);
                    // lh.selection(0, 0, 1, 0, 2);
                    lh.selection(
                        0,
                        Pos {
                            row: 0,
                            col: 1,
                            offset: 1,
                        },
                        Pos {
                            row: 0,
                            col: 2,
                            offset: 2,
                        },
                    );
                    lh
                },
                &[("a", LINE), ("b", CUR), ("c", LINE)][..],
            ),
        ];

        for (what, lh, want) in tests {
            assert_spans(lh, want, what);
        }
    }
}
