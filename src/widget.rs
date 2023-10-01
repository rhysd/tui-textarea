use crate::textarea::TextArea;
use crate::tui::buffer::Buffer;
use crate::tui::layout::Rect;
use crate::tui::text::Text;
use crate::tui::widgets::{Paragraph, Widget};
use crate::util::num_digits;
use std::cmp;
use std::sync::atomic::{AtomicU64, Ordering};

// &mut 'a (u16, u16, u16, u16) is not available since Renderer instance totally takes over the ownership of TextArea
// instance. In the case, the TextArea instance cannot be accessed from any other objects since it is mutablly
// borrowed.
//
// `tui::terminal::Frame::render_stateful_widget` would be an assumed way to render a stateful widget. But at this
// point we stick with using `tui::terminal::Frame::render_widget` because it is simpler API. Users don't need to
// manage states of textarea instances separately.
// https://docs.rs/tui/latest/tui/terminal/struct.Frame.html#method.render_stateful_widget
#[derive(Default, Debug)]
pub struct Viewport(AtomicU64);

impl Clone for Viewport {
    fn clone(&self) -> Self {
        let u = self.0.load(Ordering::Relaxed);
        Viewport(AtomicU64::new(u))
    }
}

impl Viewport {
    pub fn scroll_top(&self) -> (u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    pub fn rect(&self) -> (u16, u16, u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        let width = (u >> 48) as u16;
        let height = (u >> 32) as u16;
        let row = (u >> 16) as u16;
        let col = u as u16;
        (row, col, width, height)
    }

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

        fn next_scroll_top(prev_top: u16, cursor: u16, length: u16) -> u16 {
            if cursor < prev_top {
                cursor
            } else if prev_top + length <= cursor {
                cursor + 1 - length
            } else {
                prev_top
            }
        }

        let cursor = self.0.cursor();
        let (top_row, top_col) = self.0.viewport.scroll_top();
        let top_row = next_scroll_top(top_row, cursor.0 as u16, height);
        let top_col = next_scroll_top(top_col, cursor.1 as u16, width);

        let text = self.text(top_row as usize, height as usize);
        let mut inner = Paragraph::new(text)
            .style(self.0.style())
            .alignment(self.0.alignment());
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
