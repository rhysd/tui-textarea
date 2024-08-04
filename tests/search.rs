#![cfg(feature = "search")]

use tui_textarea::{CursorMove, TextArea};

#[test]
fn search_forward() {
    #[rustfmt::skip]
    let mut textarea = TextArea::from([
        "fooo foo",
        "foo fo foo fooo",
        "foooo",
    ]);

    // Move to 'f' on 'fo' at line 2
    textarea.move_cursor(CursorMove::Jump(1, 4));

    textarea.set_search_pattern("fo+").unwrap();

    let expected = [(1, 7), (1, 11), (2, 0), (0, 0), (0, 5), (1, 0), (1, 4)];
    for (i, pos) in expected.into_iter().enumerate() {
        let moved = textarea.search_forward(false);
        let cursor = textarea.cursor();
        assert!(moved, "{}th move didn't happen: {:?}", i, cursor);
        assert_eq!(pos, cursor, "{}th move is unexpected", i);
    }
}

#[test]
fn search_backward() {
    #[rustfmt::skip]
    let mut textarea = TextArea::from([
        "fooo foo",
        "foo fo foo fooo",
        "foooo",
    ]);

    // Move to 'f' on 'fo' at line 2
    textarea.move_cursor(CursorMove::Jump(1, 4));

    textarea.set_search_pattern("fo+").unwrap();

    let expected = [(1, 0), (0, 5), (0, 0), (2, 0), (1, 11), (1, 7), (1, 4)];
    for (i, pos) in expected.into_iter().enumerate() {
        let moved = textarea.search_back(false);
        let cursor = textarea.cursor();
        assert!(moved, "{}th move didn't happen: {:?}", i, cursor);
        assert_eq!(pos, cursor, "{}th move is unexpected", i);
    }
}

#[test]
fn search_not_found() {
    let mut textarea = TextArea::from(["fo fo fo fo"]);
    textarea.set_search_pattern("foo+").unwrap();

    assert!(!textarea.search_forward(false));
    assert!(!textarea.search_back(false));
}

#[test]
fn accept_cursor_position() {
    let mut textarea = TextArea::from(["foooo fooooooo"]);
    textarea.set_search_pattern("foo+").unwrap();

    let cursor = textarea.cursor();
    assert!(textarea.search_forward(true));
    assert_eq!(textarea.cursor(), cursor);
    assert!(textarea.search_back(true));
    assert_eq!(textarea.cursor(), cursor);
}

#[test]
fn set_search_pattern() {
    let mut textarea = TextArea::default();

    assert!(textarea.search_pattern().is_none());
    textarea.set_search_pattern("(foo").unwrap_err();
    assert!(textarea.search_pattern().is_none());

    textarea.set_search_pattern("(fo+)ba+r").unwrap();
    let pat = textarea.search_pattern().unwrap();
    assert_eq!(pat.as_str(), "(fo+)ba+r");
}
