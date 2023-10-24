#![forbid(unsafe_code)]
#![allow(clippy::needless_range_loop)]
#![warn(clippy::dbg_macro, clippy::print_stdout)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(all(
    any(
        feature = "crossterm",
        feature = "termion",
        feature = "termwiz",
        feature = "your-backend"
    ),
    any(
        feature = "tuirs-crossterm",
        feature = "tuirs-termion",
        feature = "tuirs-your-backend"
    ),
))]
compile_error!("tui-rs support and ratatui support are exclussive. only one of them can be enabled at the same time. see https://github.com/rhysd/tui-textarea#installation");

mod cursor;
mod highlight;
mod history;
mod input;
mod scroll;
#[cfg(feature = "search")]
mod search;
mod textarea;
mod util;
mod widget;
mod word;

#[cfg(any(
    feature = "crossterm",
    feature = "termion",
    feature = "termwiz",
    feature = "your-backend",
))]
#[allow(clippy::single_component_path_imports)]
use ratatui;
#[cfg(any(
    feature = "tuirs-crossterm",
    feature = "tuirs-termion",
    feature = "tuirs-your-backend",
))]
use tui as ratatui;

#[cfg(feature = "crossterm")]
#[allow(clippy::single_component_path_imports)]
use crossterm;
#[cfg(feature = "tuirs-crossterm")]
use crossterm_025 as crossterm;

pub use cursor::CursorMove;
pub use input::{Input, Key};
pub use scroll::Scrolling;
pub use textarea::TextArea;
