use crate::word::{find_word_start_backward, find_word_start_forward};
use std::cmp;

/// Specify how to move the cursor.
#[derive(Clone, Copy, Debug)]
pub enum CursorMove {
    /// Move cursor forward by one character. When the cursor is at the end of line, it moves to the head of next line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// assert_eq!(textarea.cursor(), (0, 1));
    /// textarea.move_cursor(CursorMove::Forward);
    /// assert_eq!(textarea.cursor(), (0, 2));
    /// ```
    Forward,
    /// Move cursor backward by one character. When the cursor is at the head of line, it moves to the end of previous
    /// line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Back);
    /// assert_eq!(textarea.cursor(), (0, 1));
    /// ```
    Back,
    /// Move cursor up by one line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["a", "b", "c"]);
    ///
    /// textarea.move_cursor(CursorMove::Down);
    /// textarea.move_cursor(CursorMove::Down);
    /// textarea.move_cursor(CursorMove::Up);
    /// assert_eq!(textarea.cursor(), (1, 0));
    /// ```
    Up,
    /// Move cursor down by one line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["a", "b", "c"]);
    ///
    /// textarea.move_cursor(CursorMove::Down);
    /// assert_eq!(textarea.cursor(), (1, 0));
    /// textarea.move_cursor(CursorMove::Down);
    /// assert_eq!(textarea.cursor(), (2, 0));
    /// ```
    Down,
    /// Move cursor to the head of line. When the cursor is at the head of line, it moves to the end of previous line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Head);
    /// assert_eq!(textarea.cursor(), (0, 0));
    /// ```
    Head,
    /// Move cursor to the end of line. When the cursor is at the end of line, it moves to the head of next line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::End);
    /// assert_eq!(textarea.cursor(), (0, 3));
    /// ```
    End,
    /// Move cursor to the top of lines.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["a", "b", "c"]);
    ///
    /// textarea.move_cursor(CursorMove::Down);
    /// textarea.move_cursor(CursorMove::Down);
    /// textarea.move_cursor(CursorMove::Top);
    /// assert_eq!(textarea.cursor(), (0, 0));
    /// ```
    Top,
    /// Move cursor to the bottom of lines.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["a", "b", "c"]);
    ///
    /// textarea.move_cursor(CursorMove::Bottom);
    /// assert_eq!(textarea.cursor(), (2, 0));
    /// ```
    Bottom,
    /// Move cursor forward by one word. Word boundary appears at spaces, punctuations, and others. For example
    /// `fn foo(a)` consists of words `fn`, `foo`, `(`, `a`, `)`. When the cursor is at the end of line, it moves to the
    /// head of next line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["aaa bbb ccc"]);
    ///
    /// textarea.move_cursor(CursorMove::WordForward);
    /// assert_eq!(textarea.cursor(), (0, 4));
    /// textarea.move_cursor(CursorMove::WordForward);
    /// assert_eq!(textarea.cursor(), (0, 8));
    /// ```
    WordForward,
    /// Move cursor backward by one word.  Word boundary appears at spaces, punctuations, and others. For example
    /// `fn foo(a)` consists of words `fn`, `foo`, `(`, `a`, `)`.When the cursor is at the head of line, it moves to
    /// the end of previous line.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["aaa bbb ccc"]);
    ///
    /// textarea.move_cursor(CursorMove::End);
    /// textarea.move_cursor(CursorMove::WordBack);
    /// assert_eq!(textarea.cursor(), (0, 8));
    /// textarea.move_cursor(CursorMove::WordBack);
    /// assert_eq!(textarea.cursor(), (0, 4));
    /// textarea.move_cursor(CursorMove::WordBack);
    /// assert_eq!(textarea.cursor(), (0, 0));
    /// ```
    WordBack,
    /// Move cursor down by one paragraph. Paragraph is a chunk of non-empty lines. Cursor moves to the first line of paragraph.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// // aaa
    /// //
    /// // bbb
    /// //
    /// // ccc
    /// // ddd
    /// let mut textarea = TextArea::from(["aaa", "", "bbb", "", "ccc", "ddd"]);
    ///
    /// textarea.move_cursor(CursorMove::ParagraphForward);
    /// assert_eq!(textarea.cursor(), (2, 0));
    /// textarea.move_cursor(CursorMove::ParagraphForward);
    /// assert_eq!(textarea.cursor(), (4, 0));
    /// ```
    ParagraphForward,
    /// Move cursor up by one paragraph. Paragraph is a chunk of non-empty lines. Cursor moves to the first line of paragraph.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// // aaa
    /// //
    /// // bbb
    /// //
    /// // ccc
    /// // ddd
    /// let mut textarea = TextArea::from(["aaa", "", "bbb", "", "ccc", "ddd"]);
    ///
    /// textarea.move_cursor(CursorMove::Bottom);
    /// textarea.move_cursor(CursorMove::ParagraphBack);
    /// assert_eq!(textarea.cursor(), (4, 0));
    /// textarea.move_cursor(CursorMove::ParagraphBack);
    /// assert_eq!(textarea.cursor(), (2, 0));
    /// textarea.move_cursor(CursorMove::ParagraphBack);
    /// assert_eq!(textarea.cursor(), (0, 0));
    /// ```
    ParagraphBack,
}

impl CursorMove {
    pub(crate) fn next_cursor(
        &self,
        (row, col): (usize, usize),
        lines: &[String],
    ) -> Option<(usize, usize)> {
        use CursorMove::*;

        fn fit_col(col: usize, line: &str) -> usize {
            cmp::min(col, line.chars().count())
        }

        match self {
            Forward if col >= lines[row].chars().count() => {
                (row + 1 < lines.len()).then(|| (row + 1, 0))
            }
            Forward => Some((row, col + 1)),
            Back if col == 0 => {
                let row = row.checked_sub(1)?;
                Some((row, lines[row].chars().count()))
            }
            Back => Some((row, col - 1)),
            Up => {
                let row = row.checked_sub(1)?;
                Some((row, fit_col(col, &lines[row])))
            }
            Down => (Some((row + 1, fit_col(col, lines.get(row + 1)?)))),
            Head => Some((row, 0)),
            End => Some((row, lines[row].chars().count())),
            Top => Some((0, fit_col(col, &lines[0]))),
            Bottom => {
                let row = lines.len() - 1;
                Some((row, fit_col(col, &lines[row])))
            }
            WordForward => {
                if let Some(col) = find_word_start_forward(&lines[row], col) {
                    Some((row, col))
                } else if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    Some((row, lines[row].chars().count()))
                }
            }
            WordBack => {
                if let Some(col) = find_word_start_backward(&lines[row], col) {
                    Some((row, col))
                } else if row > 0 {
                    Some((row - 1, lines[row - 1].chars().count()))
                } else {
                    Some((row, 0))
                }
            }
            ParagraphForward => {
                let mut prev_is_empty = lines[row].is_empty();
                for row in row + 1..lines.len() {
                    let line = &lines[row];
                    let is_empty = line.is_empty();
                    if !is_empty && prev_is_empty {
                        return Some((row, fit_col(col, line)));
                    }
                    prev_is_empty = is_empty;
                }
                let row = lines.len() - 1;
                Some((row, fit_col(col, &lines[row])))
            }
            ParagraphBack => {
                let row = row.checked_sub(1)?;
                let mut prev_is_empty = lines[row].is_empty();
                for row in (0..row).rev() {
                    let is_empty = lines[row].is_empty();
                    if is_empty && !prev_is_empty {
                        return Some((row + 1, fit_col(col, &lines[row + 1])));
                    }
                    prev_is_empty = is_empty;
                }
                Some((0, fit_col(col, &lines[0])))
            }
        }
    }
}
