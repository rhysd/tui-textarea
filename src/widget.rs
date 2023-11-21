use crate::cursor::DataCursor;
use crate::ratatui::buffer::Buffer;
use crate::ratatui::layout::Rect;
use crate::ratatui::text::Text;
use crate::ratatui::widgets::{Paragraph, Widget};
use crate::textarea::TextArea;
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
        let line_table = self.0.screen_lines.borrow();
        let lines_len = line_table.len();
        let lnum_len = num_digits(lines_len);
        let bottom_row = cmp::min(top_row + height, lines_len);
        let mut lines = Vec::with_capacity(bottom_row - top_row);

        if line_table.is_empty() {
            return Text::from(lines);
        }
        trace!("top_row: {}, bottom_row: {}", top_row, bottom_row);
        for (i, lp) in line_table[top_row..bottom_row].iter().enumerate() {
            trace!("line: {:?}", lp);
            let slice =
                &self.0.lines[lp.data_line][lp.byte_offset..lp.byte_length + lp.byte_offset];
            lines.push(self.0.line_spans(slice, top_row + i, lnum_len, lp));
        }
        Text::from(lines)
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        trace!(
            "DRAW {:?} =======================================================",
            area
        );
        let Rect {
            width,
            height,
            x,
            y,
        } = if let Some(b) = self.0.block() {
            b.inner(area)
        } else {
            area
        };

        // this is the first time we get to see the area we are being rendered into
        // First time in or new width, reload the line table

        if self.0.area.get().width != width {
            self.0.area.set(Rect {
                width,
                height,
                x,
                y,
            });
            self.0.screen_map_load();
        }

        fn next_scroll_top(prev_top: u16, cursor: u16, length: u16) -> u16 {
            if cursor < prev_top {
                cursor
            } else if prev_top + length <= cursor {
                cursor + 1 - length
            } else {
                prev_top
            }
        }

        let data_cursor = DataCursor(self.0.cursor().0, self.0.cursor().1);
        let screen_cursor = self.0.array_to_screen(data_cursor);
        let (top_row, top_col) = self.0.viewport.scroll_top();
        trace!(
            "top_row: {}, top_col: {}, screen_cursor: {:?} {} {}",
            top_row,
            top_col,
            screen_cursor,
            width,
            height
        );
        let top_row = next_scroll_top(top_row, screen_cursor.row as u16, height);
        let top_col = next_scroll_top(top_col, screen_cursor.col as u16, width);
        trace!(
            "top_row: {}, top_col: {}, screen_cursor: {:?}",
            top_row,
            top_col,
            screen_cursor
        );

        let (text, style) = if !self.0.placeholder.is_empty() && self.0.is_empty() {
            let text = Text::from(self.0.placeholder.as_str());
            (text, self.0.placeholder_style)
        } else {
            (self.text(top_row as usize, height as usize), self.0.style())
        };

        // To get fine control over the text color and the surrrounding block they have to be rendered separately
        // see https://github.com/ratatui-org/ratatui/issues/144
        let mut text_area = area;
        let mut inner = Paragraph::new(text)
            .style(style)
            .alignment(self.0.alignment());

        if let Some(b) = self.0.block() {
            text_area = b.inner(area);
            b.clone().render(area, buf)
        }
        if top_col != 0 {
            inner = inner.scroll((0, top_col));
        }

        // Store scroll top position for rendering on the next tick
        self.0.viewport.store(top_row, top_col, width, height);

        inner.render(text_area, buf);
        trace!("END DRAW ===================================================");
    }
}
