use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum EditKind {
    InsertChar(char, usize),
    DeleteChar(char, usize),
    InsertNewline(usize),
    DeleteNewline(usize),
    InsertStr(String, usize),
    DeleteStr(String, usize),
    InsertChunk(Vec<String>, usize, usize),
    DeleteChunk(Vec<String>, usize, usize),
}

impl EditKind {
    pub(crate) fn apply(&self, row: usize, lines: &mut Vec<String>) {
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
                lines.insert(row + 1, next_line);
            }
            EditKind::DeleteNewline(_) => {
                if row > 0 {
                    let line = lines.remove(row);
                    lines[row - 1].push_str(&line);
                }
            }
            EditKind::InsertStr(s, i) => {
                lines[row].insert_str(*i, s.as_str());
            }
            EditKind::DeleteStr(s, i) => {
                let end = *i + s.len();
                lines[row].replace_range(*i..end, "");
            }
            EditKind::InsertChunk(c, row, i) => {
                debug_assert!(c.len() > 1, "Chunk size must be > 1: {:?}", c);
                let row = *row;

                // Handle first line of chunk
                let first_line = &mut lines[row];
                let mut last_line = first_line.drain(*i..).as_str().to_string();
                first_line.push_str(&c[0]);

                // Handle last line of chunk
                last_line.insert_str(0, c.last().unwrap());
                lines.insert(row + 1, last_line);

                // Handle last line of chunk
                lines.splice(row + 1..row + 1, c[1..c.len() - 1].iter().cloned());
            }
            EditKind::DeleteChunk(c, row, i) => {
                debug_assert!(c.len() > 1, "Chunk size must be > 1: {:?}", c);
                let row = *row;

                lines[row].truncate(*i);
                let mut last_line = lines.drain(row + 1..row + c.len()).last().unwrap();
                last_line.drain(..c[c.len() - 1].len());
                lines[row].push_str(&last_line);
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
            InsertStr(s, i) => DeleteStr(s, i),
            DeleteStr(s, i) => InsertStr(s, i),
            InsertChunk(c, r, i) => DeleteChunk(c, r, i),
            DeleteChunk(c, r, i) => InsertChunk(c, r, i),
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct History {
    index: usize,
    max_items: usize,
    edits: VecDeque<Edit>,
}

impl History {
    pub fn new(max_items: usize) -> Self {
        Self {
            index: 0,
            max_items,
            edits: VecDeque::new(),
        }
    }

    pub fn push(&mut self, edit: Edit) {
        if self.max_items == 0 {
            return;
        }

        if self.edits.len() == self.max_items {
            self.edits.pop_front();
            self.index = self.index.saturating_sub(1);
        }

        if self.index < self.edits.len() {
            self.edits.truncate(self.index);
        }

        self.index += 1;
        self.edits.push_back(edit);
    }

    pub fn redo(&mut self, lines: &mut Vec<String>) -> Option<(usize, usize)> {
        if self.index == self.edits.len() {
            return None;
        }
        let edit = &self.edits[self.index];
        edit.redo(lines);
        self.index += 1;
        Some(edit.cursor_after())
    }

    pub fn undo(&mut self, lines: &mut Vec<String>) -> Option<(usize, usize)> {
        self.index = self.index.checked_sub(1)?;
        let edit = &self.edits[self.index];
        edit.undo(lines);
        Some(edit.cursor_before())
    }

    pub fn max_items(&self) -> usize {
        self.max_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_delete_chunk() {
        #[rustfmt::skip]
        let tests = [
            // Positions
            (
                // Text before edit
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                // (row, offset) position before edit
                (0, 0),
                // Chunk to be inserted
                &[
                    "x", "y",
                ][..],
                // Text after edit
                &[
                    "x",
                    "yab",
                    "cd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (0, 1),
                &[
                    "x", "y",
                ][..],
                &[
                    "ax",
                    "yb",
                    "cd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (0, 2),
                &[
                    "x", "y",
                ][..],
                &[
                    "abx",
                    "y",
                    "cd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 0),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "x",
                    "ycd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 1),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "cx",
                    "yd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 2),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "cdx",
                    "y",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (2, 0),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "cd",
                    "x",
                    "yef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (2, 1),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "cd",
                    "ex",
                    "yf",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (2, 2),
                &[
                    "x", "y",
                ][..],
                &[
                    "ab",
                    "cd",
                    "efx",
                    "y",
                ][..],
            ),
            // More than 2 lines
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 1),
                &[
                    "x", "y", "z", "w"
                ][..],
                &[
                    "ab",
                    "cx",
                    "y",
                    "z",
                    "wd",
                    "ef",
                ][..],
            ),
            // Empty lines
            (
                &[
                    "",
                    "",
                    "",
                ][..],
                (0, 0),
                &[
                    "x", "y", "z"
                ][..],
                &[
                    "x",
                    "y",
                    "z",
                    "",
                    "",
                ][..],
            ),
            (
                &[
                    "",
                    "",
                    "",
                ][..],
                (1, 0),
                &[
                    "x", "y", "z"
                ][..],
                &[
                    "",
                    "x",
                    "y",
                    "z",
                    "",
                ][..],
            ),
            (
                &[
                    "",
                    "",
                    "",
                ][..],
                (2, 0),
                &[
                    "x", "y", "z"
                ][..],
                &[
                    "",
                    "",
                    "x",
                    "y",
                    "z",
                ][..],
            ),
            // Empty buffer
            (
                &[
                    "",
                ][..],
                (0, 0),
                &[
                    "x", "y", "z"
                ][..],
                &[
                    "x",
                    "y",
                    "z",
                ][..],
            ),
            // Insert empty lines
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (0, 0),
                &[
                    "", "", "",
                ][..],
                &[
                    "",
                    "",
                    "ab",
                    "cd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 0),
                &[
                    "", "", "",
                ][..],
                &[
                    "ab",
                    "",
                    "",
                    "cd",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 1),
                &[
                    "", "", "",
                ][..],
                &[
                    "ab",
                    "c",
                    "",
                    "d",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (1, 2),
                &[
                    "", "", "",
                ][..],
                &[
                    "ab",
                    "cd",
                    "",
                    "",
                    "ef",
                ][..],
            ),
            (
                &[
                    "ab",
                    "cd",
                    "ef",
                ][..],
                (2, 2),
                &[
                    "", "", "",
                ][..],
                &[
                    "ab",
                    "cd",
                    "ef",
                    "",
                    "",
                ][..],
            ),
            // Multi-byte characters
            (
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
                (0, 0),
                &[
                    "🐷", "🐼", "🐴",
                ][..],
                &[
                    "🐷",
                    "🐼",
                    "🐴🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
            ),
            (
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
                (0, 4 * 2),
                &[
                    "🐷", "🐼", "🐴",
                ][..],
                &[
                    "🐶🐱🐷",
                    "🐼",
                    "🐴",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
            ),
            (
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
                (1, 0),
                &[
                    "🐷", "🐼", "🐴",
                ][..],
                &[
                    "🐶🐱",
                    "🐷",
                    "🐼",
                    "🐴🐮🐰",
                    "🐧🐭",
                ][..],
            ),
            (
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
                (1, 4 * 1),
                &[
                    "🐷", "🐼", "🐴",
                ][..],
                &[
                    "🐶🐱",
                    "🐮🐷",
                    "🐼",
                    "🐴🐰",
                    "🐧🐭",
                ][..],
            ),
            (
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭",
                ][..],
                (2, 4 * 2),
                &[
                    "🐷", "🐼", "🐴",
                ][..],
                &[
                    "🐶🐱",
                    "🐮🐰",
                    "🐧🐭🐷",
                    "🐼",
                    "🐴",
                ][..],
            ),
        ];

        for (before, pos, input, expected) in tests {
            let (row, offset) = pos;
            let mut lines: Vec<_> = before.iter().map(|s| s.to_string()).collect();
            let chunk: Vec<_> = input.iter().map(|s| s.to_string()).collect();

            let edit = EditKind::InsertChunk(chunk.clone(), row, offset);
            edit.apply(row, &mut lines);
            assert_eq!(
                &lines, expected,
                "{:?} at {:?} with {:?}",
                before, pos, input,
            );

            let edit = EditKind::DeleteChunk(chunk, row, offset);
            edit.apply(row, &mut lines);
            assert_eq!(
                &lines, &before,
                "{:?} at {:?} with {:?}",
                before, pos, input,
            );
        }
    }
}
