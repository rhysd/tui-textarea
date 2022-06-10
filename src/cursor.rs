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
}

impl CursorMove {
    pub fn next_cursor(
        &self,
        (row, col): (usize, usize),
        lines: &[String],
    ) -> Option<(usize, usize)> {
        fn fit_col(col: usize, line: &str) -> usize {
            let end = line.chars().count();
            if end <= col {
                end - 1
            } else {
                col
            }
        }

        match self {
            CursorMove::Forward if col + 1 >= lines[row].chars().count() => {
                if row + 1 < lines.len() {
                    Some((row + 1, 0))
                } else {
                    None
                }
            }
            CursorMove::Forward => Some((row, col + 1)),
            CursorMove::Back if col == 0 => {
                if row > 0 {
                    Some((row - 1, lines[row - 1].chars().count() - 1))
                } else {
                    None
                }
            }
            CursorMove::Back => Some((row, col - 1)),
            CursorMove::Up if row == 0 => None,
            CursorMove::Up => Some((row - 1, fit_col(col, &lines[row - 1]))),
            CursorMove::Down if row + 1 >= lines.len() => None,
            CursorMove::Down => (Some((row + 1, fit_col(col, &lines[row + 1])))),
            CursorMove::Head => Some((row, 0)),
            CursorMove::End => Some((row, lines[row].chars().count() - 1)),
            CursorMove::Top => Some((0, fit_col(col, &lines[0]))),
            CursorMove::Bottom => {
                let row = lines.len() - 1;
                Some((row, fit_col(col, &lines[row])))
            }
        }
    }
}
