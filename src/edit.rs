pub enum EditKind {
    InsertChar(char, usize),
    DeleteChar(char, usize),
    InsertNewline(usize),
    DeleteNewline(usize),
    Insert(String, usize),
}

impl EditKind {
    fn redo(&self, row: usize, lines: &mut Vec<String>) {
        match self {
            EditKind::InsertChar(c, i) => {
                lines[row].insert(*i, *c);
            }
            EditKind::DeleteChar(_, i) => {
                lines[row].remove(*i);
            }
            EditKind::InsertNewline(i) => {
                let line = &mut lines[row];
                let next_line = line[*i..].to_string();
                line.truncate(*i);
                line.push(' ');
                lines.insert(row + 1, next_line);
            }
            EditKind::DeleteNewline(_) => {
                if row > 0 {
                    let line = lines.remove(row);
                    let prev_line = &mut lines[row - 1];
                    prev_line.pop(); // Remove trailing space
                    prev_line.push_str(&line);
                }
            }
            EditKind::Insert(s, i) => {
                lines[row].insert_str(*i, s.as_str());
            }
        }
    }

    fn undo(&self, row: usize, lines: &mut Vec<String>) {
        use EditKind::*;
        match self {
            InsertChar(c, i) => DeleteChar(*c, *i).redo(row, lines),
            DeleteChar(c, i) => InsertChar(*c, *i).redo(row, lines),
            InsertNewline(i) => DeleteNewline(*i).redo(row, lines),
            DeleteNewline(i) => InsertNewline(*i).redo(row, lines),
            Insert(s, i) => {
                lines[row].replace_range(*i..s.len(), "");
            }
        }
    }
}

pub struct Edit {
    kind: EditKind,
    cursor_before: (usize, usize),
    cursor_after: (usize, usize),
}

impl Edit {
    pub fn new(
        kind: EditKind,
        cursor_before: (usize, usize),
        cursor_after: (usize, usize),
    ) -> Self {
        Self {
            kind,
            cursor_before,
            cursor_after,
        }
    }

    pub fn redo(&self, lines: &mut Vec<String>) {
        let (row, _) = self.cursor_before;
        self.kind.redo(row, lines)
    }

    pub fn undo(&self, lines: &mut Vec<String>) {
        let (row, _) = self.cursor_after;
        self.kind.undo(row, lines)
    }

    pub fn cursor_before(&self) -> (usize, usize) {
        self.cursor_before
    }

    pub fn cursor_after(&self) -> (usize, usize) {
        self.cursor_after
    }
}
