use crate::textarea::TextArea;
use crate::util::num_digits;
use std::sync::atomic::{AtomicU16, Ordering};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::text::Text;
use tui::widgets::{Block, Paragraph, Widget};

// &mut 'a (u16, u16) is not available since TextAreaWidget instance totally takes over the ownership of TextArea
// instance. In the case, the TextArea instance cannot be accessed from any other objects since it is mutablly
// borrowed.
//
// `tui::terminal::Frame::render_stateful_widget` would be an assumed way to render a stateful widget. But at this
// point we stick with using `tui::terminal::Frame::render_widget` because it is simpler API. Users don't need to
// manage states of textarea instances separately.
// https://docs.rs/tui/latest/tui/terminal/struct.Frame.html#method.render_stateful_widget
#[derive(Default)]
pub struct ScrollTop(AtomicU16, AtomicU16);

impl ScrollTop {
    fn load(&self) -> (u16, u16) {
        let row = self.0.load(Ordering::Relaxed);
        let col = self.1.load(Ordering::Relaxed);
        (row, col)
    }

    fn store(&self, row: u16, col: u16) {
        self.0.store(row, Ordering::Relaxed);
        self.1.store(col, Ordering::Relaxed);
    }
}

pub struct Renderer<'a> {
    scroll_top: &'a ScrollTop,
    cursor: (u16, u16),
    block: Option<Block<'a>>,
    inner: Paragraph<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(textarea: &'a TextArea<'a>) -> Self {
        let lnum_len = num_digits(textarea.lines().len());
        let lines: Vec<_> = textarea
            .lines()
            .iter()
            .map(String::as_str)
            .enumerate()
            .map(|(row, line)| textarea.line_spans(line, row, lnum_len))
            .collect();
        let inner = Paragraph::new(Text::from(lines)).style(textarea.style());
        let cursor = textarea.cursor();
        Self {
            scroll_top: &textarea.scroll_top,
            cursor: (cursor.0 as u16, cursor.1 as u16),
            block: textarea.block().cloned(),
            inner,
        }
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let inner_area = if let Some(b) = self.block.take() {
            let area = b.inner(area);
            self.inner = self.inner.block(b);
            area
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

        let (top_row, top_col) = self.scroll_top.load();
        let row = next_scroll_top(top_row, self.cursor.0, inner_area.height);
        let col = next_scroll_top(top_col, self.cursor.1, inner_area.width);

        let scroll = (row, col);
        if scroll != (0, 0) {
            self.inner = self.inner.scroll(scroll);
        }

        // Store scroll top position for rendering on the next tick
        self.scroll_top.store(row, col);

        self.inner.render(area, buf);
    }
}
