#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::iter;
use std::str;
use tui_textarea::{Input, Key, TextArea};

fn arbitrary_input(u: &mut Unstructured) -> Result<Input> {
    let i = u8::arbitrary(u)? % 15;
    let key = match i {
        1 => Key::Backspace,
        2 => Key::Enter,
        3 => Key::Left,
        4 => Key::Right,
        5 => Key::Up,
        6 => Key::Down,
        7 => Key::Tab,
        8 => Key::Delete,
        9 => Key::Home,
        10 => Key::End,
        11 => Key::PageUp,
        12 => Key::PageDown,
        13 => Key::Esc,
        14 => Key::Null,
        _ => match char::arbitrary(u)? {
            '\n' | '\r' => Key::Enter,
            c => Key::Char(c),
        },
    };
    Ok(Input {
        key,
        ctrl: bool::arbitrary(u)?,
        alt: bool::arbitrary(u)?,
    })
}

fn fuzz(data: &[u8]) -> Result<()> {
    let mut u = Unstructured::new(data);
    let inputs: Vec<_> = iter::repeat_with(|| arbitrary_input(&mut u))
        .take(10)
        .collect::<Result<_>>()?;
    let text = <&str>::arbitrary(&mut u)?;
    let mut textarea = TextArea::from(text.lines());
    for input in inputs {
        textarea.input(input);
    }
    let _ = textarea.widget();
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = fuzz(data);
});
