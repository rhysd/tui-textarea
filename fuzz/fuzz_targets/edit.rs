#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::iter;
use std::str;
use tui_textarea::{Input, TextArea};
use tui_textarea_bench::{dummy_terminal, TerminalExt};

fn fuzz(data: &[u8]) -> Result<()> {
    let mut term = dummy_terminal();
    let mut data = Unstructured::new(data);
    let inputs: Vec<_> = iter::repeat_with(|| Input::arbitrary(&mut data))
        .take(100)
        .collect::<Result<_>>()?;
    let text = <&str>::arbitrary(&mut data)?;
    let mut textarea = TextArea::from(text.lines());
    for input in inputs {
        textarea.input(input);
        term.draw_textarea(&mut textarea);
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = fuzz(data);
});
