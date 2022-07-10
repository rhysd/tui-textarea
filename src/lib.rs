#![allow(clippy::needless_range_loop)]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![doc = include_str!("../README.md")]

mod cursor;
mod highlight;
mod history;
mod input;
#[cfg(feature = "search")]
mod search;
mod textarea;
mod util;
mod word;

pub use cursor::CursorMove;
pub use input::{Input, Key};
pub use textarea::TextArea;
