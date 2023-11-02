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

#[test]
fn test_insert_str_one_line() {
    for i in 0..="ab".len() {
        let mut t = TextArea::from(["ab"]);
        t.move_cursor(CursorMove::Jump(0, i as u16));
        assert!(t.insert_str("x"), "{}", i);
        let have = &t.lines()[0];

        let mut want = "ab".to_string();
        want.insert(i, 'x');
        assert_eq!(&want, have, "{}", i);
    }

    let mut t = TextArea::default();
    assert!(t.insert_str("x"));
    assert_eq!(t.lines(), ["x"]);
}

#[test]
fn test_insert_str_empty_line() {
    let mut t = TextArea::from(["ab"]);
    assert!(!t.insert_str(""));
    assert_eq!(t.lines(), ["ab"]);
}

#[test]
fn test_insert_str_multiple_lines() {
    #[rustfmt::skip]
    let tests = [
        // Positions
        (
            // Text before edit
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            // (row, offset) position before edit
            (0, 0),
            // String to be inserted
            "x\ny",
            // (row, offset) position after edit
            (1, 1),
            // Text after edit
            &[
                "x",
                "yab",
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 1),
            "x\ny",
            (1, 1),
            &[
                "ax",
                "yb",
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 2),
            "x\ny",
            (1, 1),
            &[
                "abx",
                "y",
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 0),
            "x\ny",
            (2, 1),
            &[
                "ab",
                "x",
                "ycd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 1),
            "x\ny",
            (2, 1),
            &[
                "ab",
                "cx",
                "yd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 2),
            "x\ny",
            (2, 1),
            &[
                "ab",
                "cdx",
                "y",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 0),
            "x\ny",
            (3, 1),
            &[
                "ab",
                "cd",
                "x",
                "yef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 1),
            "x\ny",
            (3, 1),
            &[
                "ab",
                "cd",
                "ex",
                "yf",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 2),
            "x\ny",
            (3, 1),
            &[
                "ab",
                "cd",
                "efx",
                "y",
            ][..],
        ),
        // More than 2 lines
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 1),
            "x\ny\nz\nw",
            (4, 1),
            &[
                "ab",
                "cx",
                "y",
                "z",
                "wd",
                "ef",
            ][..],
        ),
        // Empty lines
        (
            &[
                "",
                "",
                "",
            ][..],
            (0, 0),
            "x\ny\nz",
            (2, 1),
            &[
                "x",
                "y",
                "z",
                "",
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (1, 0),
            "x\ny\nz",
            (3, 1),
            &[
                "",
                "x",
                "y",
                "z",
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (2, 0),
            "x\ny\nz",
            (4, 1),
            &[
                "",
                "",
                "x",
                "y",
                "z",
            ][..],
        ),
        // Empty buffer
        (
            &[
                "",
            ][..],
            (0, 0),
            "x\ny\nz",
            (2, 1),
            &[
                "x",
                "y",
                "z",
            ][..],
        ),
        // Insert empty lines
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            "\n\n\n",
            (2, 0),
            &[
                "",
                "",
                "ab",
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 0),
            "\n\n\n",
            (3, 0),
            &[
                "ab",
                "",
                "",
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 1),
            "\n\n\n",
            (3, 0),
            &[
                "ab",
                "c",
                "",
                "d",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 2),
            "\n\n\n",
            (3, 0),
            &[
                "ab",
                "cd",
                "",
                "",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 2),
            "\n\n\n",
            (4, 0),
            &[
                "ab",
                "cd",
                "ef",
                "",
                "",
            ][..],
        ),
        // Multi-byte characters
        (
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
            (0, 0),
            "🐷\n🐼\n🐴",
            (2, 1),
            &[
                "🐷",
                "🐼",
                "🐴🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
        ),
        (
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
            (0, 2),
            "🐷\n🐼\n🐴",
            (2, 1),
            &[
                "🐶🐱🐷",
                "🐼",
                "🐴",
                "🐮🐰",
                "🐧🐭",
            ][..],
        ),
        (
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
            (1, 0),
            "🐷\n🐼\n🐴",
            (3, 1),
            &[
                "🐶🐱",
                "🐷",
                "🐼",
                "🐴🐮🐰",
                "🐧🐭",
            ][..],
        ),
        (
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
            (1, 1),
            "🐷\n🐼\n🐴",
            (3, 1),
            &[
                "🐶🐱",
                "🐮🐷",
                "🐼",
                "🐴🐰",
                "🐧🐭",
            ][..],
        ),
        (
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭",
            ][..],
            (2, 2),
            "🐷\n🐼\n🐴",
            (4, 1),
            &[
                "🐶🐱",
                "🐮🐰",
                "🐧🐭🐷",
                "🐼",
                "🐴",
            ][..],
        ),
    ];

    for test in tests {
        let (before, before_pos, input, after_pos, expected) = test;

        let mut t = TextArea::from(before.iter().map(|s| s.to_string()));
        let (row, col) = before_pos;
        t.move_cursor(CursorMove::Jump(row, col));

        assert!(t.insert_str(input), "{:?}", test);
        assert_eq!(t.cursor(), after_pos, "{:?}", test);
        assert_eq!(t.lines(), expected, "{:?}", test);

        assert!(t.undo(), "undo: {:?}", test);
        assert_eq!(t.lines(), before, "content after undo: {:?}", test);
        let before_pos = (row as _, col as _);
        assert_eq!(t.cursor(), before_pos, "cursor after undo: {:?}", test);
    }
}

#[test]
fn test_delete_str_nothing() {
    for i in 0..="ab".len() {
        let mut t = TextArea::from(["ab"]);
        assert!(!t.delete_str(0), "{}", i);
    }
    let mut t = TextArea::default();
    assert!(!t.delete_str(0));
}

#[test]
fn test_delete_str_within_line() {
    for i in 0.."abc".len() {
        for j in 1..="abc".len() - i {
            let mut t = TextArea::from(["abc"]);
            t.move_cursor(CursorMove::Jump(0, i as _));
            assert!(t.delete_str(j), "at {}, size={}", i, j);
            let have = &t.lines()[0];

            let mut want = "abc".to_string();
            want.drain(i..i + j);
            assert_eq!(&want, have, "at {}, size={}", i, j);
        }
    }
}

#[test]
fn test_delete_str_multiple_lines() {
    #[rustfmt::skip]
    let tests = [
        // Length
        (
            // Text before edit
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            // (row, offset) cursor position
            (0, 0),
            // Chars to be deleted
            3,
            // Deleted text
            "ab\n",
            // Text after edit
            &[
                "cd",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            4,
            "ab\nc",
            &[
                "d",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            5,
            "ab\ncd",
            &[
                "",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            6,
            "ab\ncd\n",
            &[
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            7,
            "ab\ncd\ne",
            &[
                "f",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            8,
            "ab\ncd\nef",
            &[
                "",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            9,
            "ab\ncd\nef",
            &[
                "",
            ][..],
        ),
        // Positions
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 1),
            3,
            "b\nc",
            &[
                "ad",
                "ef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 2),
            4,
            "\ncd\n",
            &[
                "abef",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 0),
            4,
            "cd\ne",
            &[
                "ab",
                "f",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 0),
            3,
            "ef",
            &[
                "ab",
                "cd",
                "",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 1),
            2,
            "f",
            &[
                "ab",
                "cd",
                "e",
            ][..],
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (2, 2),
            1,
            "",
            &[
                "ab",
                "cd",
                "ef",
            ][..],
        ),
        // Empty lines
        (
            &[
                "",
                "",
                "",
            ][..],
            (0, 0),
            1,
            "\n",
            &[
                "",
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (0, 0),
            2,
            "\n\n",
            &[
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (0, 0),
            3,
            "\n\n",
            &[
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (1, 0),
            1,
            "\n",
            &[
                "",
                "",
            ][..],
        ),
        (
            &[
                "",
                "",
                "",
            ][..],
            (2, 0),
            1,
            "",
            &[
                "",
                "",
                "",
            ][..],
        ),
        // Empty buffer
        (
            &[
                "",
            ][..],
            (0, 0),
            1,
            "",
            &[
                "",
            ][..],
        ),
    ];

    for test in tests {
        let (before, (row, col), chars, deleted, after) = test;

        let mut t = TextArea::from(before.iter().map(|s| s.to_string()));
        t.move_cursor(CursorMove::Jump(row as _, col as _));

        assert!(t.delete_str(chars), "did not modified: {:?}", test);
        assert_eq!(t.cursor(), (row, col), "cursor position: {:?}", test);
        assert_eq!(t.lines(), after, "text buffer content: {:?}", test);
        assert_eq!(t.yank_text(), deleted, "yanked text: {:?}", test);

        assert!(t.undo(), "undo: {:?}", test);
        assert_eq!(t.lines(), before, "content after undo: {:?}", test);
        assert_eq!(t.cursor(), (row, col), "cursor after undo: {:?}", test);
    }
}
