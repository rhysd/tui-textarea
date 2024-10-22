#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::str;
use tui_textarea::{CursorMove, Input, TextArea};
use tui_textarea_bench::{dummy_terminal, TerminalExt};

#[derive(Arbitrary)]
enum RandomInput {
    Input(Input),
    Cursor(CursorMove),
}

impl RandomInput {
    fn apply(self, t: &mut TextArea<'_>) {
        match self {
            Self::Input(input) => {
                t.input(input);
            }
            Self::Cursor(m) => t.move_cursor(m),
        }
    }
}

fn fuzz(data: &[u8]) -> Result<()> {
    let mut term = dummy_terminal();
    let mut data = Unstructured::new(data);
    let text = <&str>::arbitrary(&mut data)?;
    let mut textarea = TextArea::from(text.lines());
    for _ in 0..100 {
        let input = RandomInput::arbitrary(&mut data)?;
        input.apply(&mut textarea);
        term.draw_textarea(&textarea);
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = fuzz(data);
});
