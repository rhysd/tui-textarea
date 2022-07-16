use crate::textarea::TextArea;
use crate::util::num_digits;
use std::sync::atomic::{AtomicU32, Ordering};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::{Paragraph, Widget};

// &mut 'a (u16, u16) is not available since TextAreaWidget instance totally takes over the ownership of TextArea
// instance. In the case, the TextArea instance cannot be accessed from any other objects since it is mutablly
// borrowed.
//
// `tui::terminal::Frame::render_stateful_widget` would be an assumed way to render a stateful widget. But at this
// point we stick with using `tui::terminal::Frame::render_widget` because it is simpler API. Users don't need to
// manage states of textarea instances separately.
// https://docs.rs/tui/latest/tui/terminal/struct.Frame.html#method.render_stateful_widget
#[derive(Default)]
pub struct ScrollTop(AtomicU32);

impl Clone for ScrollTop {
    fn clone(&self) -> Self {
        let u = self.0.load(Ordering::Relaxed);
        ScrollTop(AtomicU32::new(u))
    }
}

impl ScrollTop {
    fn load(&self) -> (u16, u16) {
        let u = self.0.load(Ordering::Relaxed);
        ((u >> 16) as u16, u as u16)
    }

    fn store(&self, row: u16, col: u16) {
        let u = ((row as u32) << 16) | col as u32;
        self.0.store(u, Ordering::Relaxed);
    }
}

pub struct Renderer<'a>(&'a TextArea<'a>);

impl<'a> Renderer<'a> {
    pub fn new(textarea: &'a TextArea<'a>) -> Self {
        Self(textarea)
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = if let Some(b) = self.0.block() {
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
        let (top_row, top_col) = self.0.scroll_top.load();
        let row = next_scroll_top(top_row, cursor.0 as u16, inner_area.height);
        let col = next_scroll_top(top_col, cursor.1 as u16, inner_area.width);

        let lnum_len = num_digits(self.0.lines().len());
        let lines: Vec<_> = self
            .0
            .lines()
            .iter()
            .map(String::as_str)
            .enumerate()
            .skip(row as usize)
            .take(inner_area.height as usize)
            .map(|(row, line)| self.0.line_spans(line, row, lnum_len))
            .collect();

        let mut inner = Paragraph::new(Text::from(lines)).style(self.0.style());
        if let Some(b) = self.0.block() {
            inner = inner.block(b.clone());
        }
        if col != 0 {
            inner = inner.scroll((0, col));
        }

        // Store scroll top position for rendering on the next tick
        self.0.scroll_top.store(row, col);

        inner.render(area, buf);
    }
}
