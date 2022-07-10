// We use empty backend for our benchmark instead of tui::backend::TestBackend to make impact of benchmark from tui-rs
// as small as possible.

use std::io;
use tui::backend::Backend;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::Terminal;
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
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            cursor: (0, 0),
        }
    }
}

impl Backend for DummyBackend {
    fn draw<'a, I>(&mut self, _content: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    fn get_cursor(&mut self) -> Result<(u16, u16), io::Error> {
        Ok(self.cursor)
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), io::Error> {
        self.cursor = (x, y);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    fn size(&self) -> Result<Rect, io::Error> {
        Ok(Rect {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        })
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

pub fn dummy_terminal() -> Terminal<DummyBackend> {
    Terminal::new(DummyBackend::default()).unwrap()
}

pub trait TerminalExt {
    fn draw_textarea(&mut self, textarea: &TextArea<'_>);
}

impl TerminalExt for Terminal<DummyBackend> {
    fn draw_textarea(&mut self, textarea: &TextArea<'_>) {
        self.draw(|f| f.render_widget(textarea.widget(), f.size()))
            .unwrap();
    }
}
