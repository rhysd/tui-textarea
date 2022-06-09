#[derive(Clone)]
pub enum EditKind {
    InsertChar(char, usize),
    DeleteChar(char, usize),
    InsertNewline(usize),
    DeleteNewline(usize),
    Insert(String, usize),
    Remove(String, usize),
}

impl EditKind {
    fn apply(&self, row: usize, lines: &mut Vec<String>) {
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
            EditKind::Remove(s, i) => {
                let end = *i + s.len();
                lines[row].replace_range(*i..end, "");
            }
        }
    }

    fn invert(&self) -> Self {
        use EditKind::*;
        match self.clone() {
            InsertChar(c, i) => DeleteChar(c, i),
            DeleteChar(c, i) => InsertChar(c, i),
            InsertNewline(i) => DeleteNewline(i),
            DeleteNewline(i) => InsertNewline(i),
            Insert(s, i) => Remove(s, i),
            Remove(s, i) => Insert(s, i),
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
        self.kind.apply(row, lines);
    }

    pub fn undo(&self, lines: &mut Vec<String>) {
        let (row, _) = self.cursor_after;
        self.kind.invert().apply(row, lines); // Undo is redo of inverted edit
    }

    pub fn cursor_before(&self) -> (usize, usize) {
        self.cursor_before
    }

    pub fn cursor_after(&self) -> (usize, usize) {
        self.cursor_after
    }
}
