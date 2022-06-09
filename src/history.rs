use crate::edit::Edit;
use std::collections::VecDeque;

pub struct EditHistory {
    index: usize,
    max_items: usize,
    edits: VecDeque<Edit>,
}

impl EditHistory {
    pub fn new(max_items: usize) -> Self {
        Self {
            index: 0,
            max_items,
            edits: VecDeque::new(),
        }
    }

    pub fn push(&mut self, edit: Edit) {
        if self.edits.len() == self.max_items {
            self.edits.pop_front();
            self.index -= 1;
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
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        let edit = &self.edits[self.index];
        edit.undo(lines);
        Some(edit.cursor_before())
    }
}
