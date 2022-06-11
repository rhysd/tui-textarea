use crate::word::{find_word_start_backward, find_word_start_forward};
use std::cmp;

#[derive(Clone, Copy, Debug)]
pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Head,
    End,
    Top,
    Bottom,
    WordForward,
    WordBack,
    ParagraphForward,
    ParagraphBack,
}

impl CursorMove {
    pub fn next_cursor(
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
                if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    None
                }
            }
            Forward => Some((row, col + 1)),
            Back if col == 0 => {
                if row > 0 {
                    Some((row - 1, lines[row - 1].chars().count()))
                } else {
                    None
                }
            }
            Back => Some((row, col - 1)),
            Up if row == 0 => None,
            Up => Some((row - 1, fit_col(col, &lines[row - 1]))),
            Down if row + 1 >= lines.len() => None,
            Down => (Some((row + 1, fit_col(col, &lines[row + 1])))),
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
            ParagraphBack if row == 0 => None,
            ParagraphBack => {
                let mut prev_is_empty = lines[row - 1].is_empty();
                for row in (0..row - 1).rev() {
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
