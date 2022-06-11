#![allow(clippy::needless_range_loop)]

mod cursor;
mod history;
mod input;
mod textarea;

pub use cursor::CursorMove;
pub use input::{Input, Key};
pub use textarea::TextArea;
