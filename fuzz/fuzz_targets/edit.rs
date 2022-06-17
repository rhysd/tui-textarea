#![no_main]

use arbitrary::{Arbitrary, Result, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::iter;
use std::str;
use tui_textarea::{Input, Key, TextArea};

macro_rules! arbitrary_key_enum {
    ($($p:ident$(($x:ident))?,)+) => {

        #[derive(Arbitrary)]
        enum ArbitraryKey {
            $(
                $p$(($x))?,
            )+
        }

        impl From<ArbitraryKey> for Key {
            fn from(k: ArbitraryKey) -> Key {
                match k {
                    $(
                        ArbitraryKey::$p$(($x))? => Key::$p$(($x))?,
                    )+
                }
            }
        }
    }
}

arbitrary_key_enum!(
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,
    Null,
);

#[derive(Arbitrary)]
struct ArbitraryInput {
    key: ArbitraryKey,
    ctrl: bool,
    alt: bool,
}

impl From<ArbitraryInput> for Input {
    fn from(i: ArbitraryInput) -> Input {
        Input {
            key: i.key.into(),
            ctrl: i.ctrl,
            alt: i.alt,
        }
    }
}

fn fuzz(data: &[u8]) -> Result<()> {
    let mut u = Unstructured::new(data);
    let inputs: Vec<_> = iter::repeat_with(|| ArbitraryInput::arbitrary(&mut u))
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
