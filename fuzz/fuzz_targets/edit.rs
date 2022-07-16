#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::iter;
use std::str;
use tui_textarea::{Input, TextArea};

fn fuzz(data: &[u8]) -> Result<()> {
    let mut u = Unstructured::new(data);
    let inputs: Vec<_> = iter::repeat_with(|| Input::arbitrary(&mut u))
        .take(100)
        .collect::<Result<_>>()?;
    let text = <&str>::arbitrary(&mut u)?;
    let mut textarea = TextArea::from(text.lines());
    for input in inputs {
        textarea.input(input);
        let _ = textarea.widget();
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = fuzz(data);
});
