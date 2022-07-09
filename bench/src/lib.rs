// We use empty backend for our benchmark instead of tui::backend::TestBackend to make impact of benchmark from tui-rs
// as small as possible.

use std::io;
use tui::backend::Backend;
use tui::buffer::Cell;
use tui::layout::Rect;
use tui::Terminal;

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
