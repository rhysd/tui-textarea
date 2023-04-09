use crate::textarea::TextArea;
use crate::util::num_digits;
use std::cmp;
use std::sync::atomic::{AtomicU64, Ordering};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::{Paragraph, Widget, Wrap};

// &mut 'a (u16, u16, u16, u16) is not available since Renderer instance totally takes over the ownership of TextArea
// instance. In the case, the TextArea instance cannot be accessed from any other objects since it is mutablly
// borrowed.
//
// `tui::terminal::Frame::render_stateful_widget` would be an assumed way to render a stateful widget. But at this
// point we stick with using `tui::terminal::Frame::render_widget` because it is simpler API. Users don't need to
// manage states of textarea instances separately.
// https://docs.rs/tui/latest/tui/terminal/struct.Frame.html#method.render_stateful_widget
#[derive(Default)]
pub struct Viewport(AtomicU64);

impl Clone for Viewport {
    fn clone(&self) -> Self {
        let u = self.0.load(Ordering::Relaxed);
        Viewport(AtomicU64::new(u))
    }
}

impl Viewport {
    // Return coordinates at top of viewport
    pub fn scroll_top(&self) -> (u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    // Return scroll top position, and width / height
    pub fn rect(&self) -> (u16, u16, u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        let width = (u >> 48) as u16;
        let height = (u >> 32) as u16;
        let row = (u >> 16) as u16;
        let col = u as u16;
        (row, col, width, height)
    }

    // What is the difference
    pub fn position(&self) -> (u16, u16, u16, u16) {
        let (row_top, col_top, width, height) = self.rect();
        let row_bottom = row_top.saturating_add(height).saturating_sub(1);
        let col_bottom = col_top.saturating_add(width).saturating_sub(1);

        (
            row_top,
            col_top,
            cmp::max(row_top, row_bottom),
            cmp::max(col_top, col_bottom),
        )
    }

    fn store(&self, row: u16, col: u16, width: u16, height: u16) {
        // Pack four u16 values into one u64 value
        let u =
            ((width as u64) << 48) | ((height as u64) << 32) | ((row as u64) << 16) | col as u64;
        self.0.store(u, Ordering::Relaxed);
    }

    pub fn scroll(&mut self, rows: i16, cols: i16) {
        fn apply_scroll(pos: u16, delta: i16) -> u16 {
            if delta >= 0 {
                pos.saturating_add(delta as u16)
            } else {
                pos.saturating_sub(-delta as u16)
            }
        }

        let u = self.0.get_mut();
        let row = apply_scroll((*u >> 16) as u16, rows);
        let col = apply_scroll(*u as u16, cols);
        *u = (*u & 0xffff_ffff_0000_0000) | ((row as u64) << 16) | (col as u64);
    }
}

pub struct Renderer<'a>(&'a TextArea<'a>);

impl<'a> Renderer<'a> {
    pub fn new(textarea: &'a TextArea<'a>) -> Self {
        Self(textarea)
    }

    #[inline]
    fn text(&self, top_row: usize, height: usize) -> Text<'a> {
        let lines_len = self.0.lines().len();
        let lnum_len = num_digits(lines_len);
        let bottom_row = cmp::min(top_row + height, lines_len);
        let mut lines = Vec::with_capacity(bottom_row - top_row);
        for (i, line) in self.0.lines()[top_row..bottom_row].iter().enumerate() {
            lines.push(self.0.line_spans(line.as_str(), top_row + i, lnum_len));
        }
        Text::from(lines)
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Rect { width, height, .. } = if let Some(b) = self.0.block() {
            b.inner(area)
        } else {
            area
        };

        // Transform lines into array of row count for each line
        fn wrapped_rows(lines: &[String], wrap_width: u16) -> Vec<u16> {
            lines
                .iter()
                .map(|line| {
                    let line_length = line.chars().count() as u16;
                    // Empty rows occupy at least 1 row
                    ((line_length + wrap_width - 1) / wrap_width).max(1)
                })
                .collect()
        }

        // Move scroll top if cursor moves outside of viewport
        // Cursor position is relative to text lines, not viewport.
        fn next_scroll_top(prev_top: u16, cursor: u16, length: u16) -> u16 {
            if cursor < prev_top {
                cursor
            } else if prev_top + length <= cursor {
                cursor + 1 - length
            } else {
                prev_top
            }
        }

        fn next_scroll_row_wrapped(
            prev_top_row: u16,
            cursor_row: u16,
            viewport_height: u16,
            wrapped_rows: &Vec<u16>,
        ) -> u16 {
            if cursor_row < prev_top_row {
                return cursor_row;
            } else {
                // Calculate the number of wrap rows between the top row and the cursor row
                let rows_from_top_to_cursor = wrapped_rows
                    [prev_top_row as usize..cursor_row as usize]
                    .iter()
                    .sum::<u16>();
                let cursor_row_wraps = wrapped_rows[cursor_row as usize] - 1;
                let cursor_line_on_screen =
                    rows_from_top_to_cursor + cursor_row_wraps <= viewport_height;
                let rows_to_move = rows_from_top_to_cursor + cursor_row_wraps - viewport_height;

                if !cursor_line_on_screen {
                    // Count how many lines add up to enough rows to get entire cursor line on screen again
                    let lines_to_move = wrapped_rows[prev_top_row as usize..cursor_row as usize]
                        .iter()
                        .scan(0, |acc, &row| {
                            // Sum wrap rows to this line
                            *acc += row;
                            Some(*acc)
                        })
                        // Return index of line where acc exceeds rows_to_move
                        .position(|sum| sum >= rows_to_move)
                        .unwrap_or(0) as u16;

                    // Never move below cursor row in case terminal can't fit it
                    return (prev_top_row + lines_to_move).min(cursor_row);
                } else {
                    return prev_top_row;
                }
            };
        }

        let cursor = self.0.cursor();
        let (mut top_row, mut top_col) = self.0.viewport.scroll_top();
        let wrap = self.0.get_wrap();
        if wrap {
            let wrapped_rows = wrapped_rows(&self.0.lines(), width);
            top_row = next_scroll_row_wrapped(top_row, cursor.0 as u16, height, &wrapped_rows);
            // Column for scoll should never change with wrapping (no horiz scroll)
        } else {
            top_row = next_scroll_top(top_row, cursor.0 as u16, height);
            top_col = next_scroll_top(top_col, cursor.1 as u16, width);
        }
        let (top_row, top_col) = (top_row, top_col);

        let text = self.text(top_row as usize, height as usize);
        let mut inner = Paragraph::new(text)
            .style(self.0.style())
            .alignment(self.0.alignment());
        if wrap {
            inner = inner.wrap(Wrap { trim: false });
        }
        if let Some(b) = self.0.block() {
            inner = inner.block(b.clone());
        }
        if top_col != 0 {
            inner = inner.scroll((0, top_col));
        }

        // Store scroll top position for rendering on the next tick
        self.0.viewport.store(top_row, top_col, width, height);

        inner.render(area, buf);
    }
}
