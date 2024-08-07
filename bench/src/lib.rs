// We use empty backend for our benchmark instead of tui::backend::TestBackend to make impact of benchmark from tui-rs
// as small as possible.

use ratatui::backend::{Backend, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::Terminal;
use std::io;
use tui_textarea::TextArea;

pub const LOREM: &[&str] = &[
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do",
    "eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim",
    "ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut",
    "aliquip ex ea commodo consequat. Duis aute irure dolor in",
    "reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla",
    "pariatur. Excepteur sint occaecat cupidatat non proident, sunt in",
    "culpa qui officia deserunt mollit anim id est laborum.",
];
pub const SEED: [u8; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
];

pub struct DummyBackend {
    width: u16,
    height: u16,
    cursor: (u16, u16),
}

impl Default for DummyBackend {
    #[inline]
    fn default() -> Self {
        Self {
            width: 40,
            height: 12,
            cursor: (0, 0),
        }
    }
}

impl Backend for DummyBackend {
    #[inline]
    fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    #[inline]
    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        Ok(self.cursor)
    }

    #[inline]
    fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.cursor = (x, y);
        Ok(())
    }

    #[inline]
    fn clear(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn size(&self) -> io::Result<Size> {
        Ok(Size {
            width: self.width,
            height: self.height,
        })
    }

    #[inline]
    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size {
                width: self.width,
                height: self.height,
            },
            pixels: Size {
                width: self.width * 6,
                height: self.height * 12,
            },
        })
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn get_cursor_position(&mut self) -> io::Result<Position> {
        let (x, y) = self.cursor;
        Ok(Position { x, y })
    }

    #[inline]
    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        let Position { x, y } = position.into();
        self.cursor = (x, y);
        Ok(())
    }
}

#[inline]
pub fn dummy_terminal() -> Terminal<DummyBackend> {
    Terminal::new(DummyBackend::default()).unwrap()
}

pub trait TerminalExt {
    fn draw_textarea(&mut self, textarea: &TextArea<'_>);
}

impl TerminalExt for Terminal<DummyBackend> {
    #[inline]
    fn draw_textarea(&mut self, textarea: &TextArea<'_>) {
        self.draw(|f| f.render_widget(textarea, f.area())).unwrap();
    }
}
