use tui_textarea::{CursorMove, TextArea};

#[test]
fn test_insert_soft_tab() {
    for test in [
        ("", 0, "    "),
        ("a", 1, "a   "),
        ("abcd", 4, "abcd    "),
        ("a", 0, "    a"),
        ("ab", 1, "a   b"),
        ("abcdefgh", 4, "abcd    efgh"),
        ("あ", 1, "あ  "),
        ("🐶", 1, "🐶  "),
        ("あ", 0, "    あ"),
        ("あい", 1, "あ  い"),
    ] {
        let (input, col, expected) = test;
        let mut t = TextArea::from([input.to_string()]);
        t.move_cursor(CursorMove::Jump(0, col));
        assert!(t.insert_tab(), "{:?}", test);
        let lines = t.into_lines();
        assert_eq!(lines.len(), 1, "{:?}, {:?}", lines, test);
        let line = &lines[0];
        assert_eq!(line, expected, "{:?}", test);
    }
}
