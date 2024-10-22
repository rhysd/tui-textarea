#![no_main]

use arbitrary::{Arbitrary as _, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use tui_textarea::{CursorMove, TextArea};
use tui_textarea_bench::{dummy_terminal, TerminalExt};

fn fuzz(data: &[u8]) -> Result<()> {
    let mut term = dummy_terminal();
    let mut textarea = TextArea::default();
    let mut data = Unstructured::new(data);
    for i in 0..100 {
        textarea.move_cursor(CursorMove::arbitrary(&mut data)?);
        if i % 2 == 0 {
            textarea.insert_str(String::arbitrary(&mut data)?);
        } else {
            textarea.delete_str(usize::arbitrary(&mut data)?);
        }
        term.draw_textarea(&textarea);
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    fuzz(data).unwrap();
});
