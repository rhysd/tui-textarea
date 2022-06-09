#[derive(Clone, Copy, Debug)]
pub enum CursorMove {
    Forward,
    Back,
    Up,
    Down,
    Head,
    End,
}

impl CursorMove {
    pub fn next_cursor(
        &self,
        (row, col): (usize, usize),
        lines: &[String],
    ) -> Option<(usize, usize)> {
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
            CursorMove::Up => {
                let mut col = col;
                let end = lines[row - 1].chars().count();
                if end <= col {
                    col = end - 1;
                }
                Some((row - 1, col))
            }
            CursorMove::Down if row + 1 >= lines.len() => None,
            CursorMove::Down => {
                let mut col = col;
                let end = lines[row + 1].chars().count();
                if end <= col {
                    col = end - 1;
                }
                Some((row + 1, col))
            }
            CursorMove::Head => Some((row, 0)),
            CursorMove::End => Some((row, lines[row].chars().count() - 1)),
        }
    }
}
