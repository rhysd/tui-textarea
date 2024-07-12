#![cfg(feature = "serde")]

use tui_textarea::{CursorMove, Input, Key, Scrolling};

#[test]
fn test_serde_key() {
    let k = Key::Char('a');
    let s = serde_json::to_string(&k).unwrap();
    assert_eq!(s, r#"{"Char":"a"}"#);
    let d: Key = serde_json::from_str(&s).unwrap();
    assert_eq!(d, k);
}

#[test]
fn test_serde_input() {
    let i = Input {
        key: Key::Char('a'),
        ctrl: true,
        alt: false,
        shift: true,
    };
    let s = serde_json::to_string(&i).unwrap();
    assert_eq!(
        s,
        r#"{"key":{"Char":"a"},"ctrl":true,"alt":false,"shift":true}"#,
    );
    let d: Input = serde_json::from_str(&s).unwrap();
    assert_eq!(d, i);
}

#[test]
fn test_serde_scrolling() {
    let scroll = Scrolling::Delta { rows: 1, cols: 2 };
    let s = serde_json::to_string(&scroll).unwrap();
    assert_eq!(s, r#"{"Delta":{"rows":1,"cols":2}}"#);
    let d: Scrolling = serde_json::from_str(&s).unwrap();
    assert_eq!(d, scroll);
}

#[test]
fn test_serde_cursor_move() {
    let c = CursorMove::Forward;
    let s = serde_json::to_string(&c).unwrap();
    assert_eq!(s, r#""Forward""#);
    let d: CursorMove = serde_json::from_str(&s).unwrap();
    assert_eq!(d, c);
}
