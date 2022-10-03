#![allow(clippy::needless_range_loop)]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod cursor;
mod highlight;
mod history;
mod input;
#[cfg(feature = "search")]
mod search;
mod textarea;
mod util;
mod widget;
mod word;

pub use cursor::CursorMove;
pub use input::{Input, Key};
pub use textarea::{Scrolling, TextArea};
