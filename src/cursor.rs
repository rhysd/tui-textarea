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

#[derive(PartialEq, Eq, Clone, Copy)]
enum CharKind {
    Space,
    Punct,
    Other,
}

impl CharKind {
    fn new(c: char) -> Self {
        if c.is_whitespace() {
            Self::Space
        } else if c.is_ascii_punctuation() {
            Self::Punct
        } else {
            Self::Other
        }
    }
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
                fn find_word_forward(line: &str, col: usize) -> Option<usize> {
                    let mut it = line.chars().enumerate().skip(col);
                    let mut prev = CharKind::new(it.next()?.1);
                    for (col, c) in it {
                        let cur = CharKind::new(c);
                        if cur != CharKind::Space && prev != cur {
                            return Some(col);
                        }
                        prev = cur;
                    }
                    None
                }
                if let Some(col) = find_word_forward(&lines[row], col) {
                    Some((row, col))
                } else if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    Some((row, lines[row].chars().count()))
                }
            }
            WordBack => {
                fn find_word_back(line: &str, col: usize) -> Option<usize> {
                    let idx = line
                        .char_indices()
                        .nth(col)
                        .map(|(i, _)| i)
                        .unwrap_or(line.len());
                    let mut it = line[..idx].chars().rev().enumerate();
                    let mut cur = CharKind::new(it.next()?.1);
                    for (i, c) in it {
                        let next = CharKind::new(c);
                        if cur != CharKind::Space && next != cur {
                            return Some(col - i);
                        }
                        cur = next;
                    }
                    if cur != CharKind::Space {
                        Some(0)
                    } else {
                        None
                    }
                }
                if let Some(col) = find_word_back(&lines[row], col) {
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
