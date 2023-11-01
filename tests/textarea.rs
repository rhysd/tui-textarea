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
    ];

    for test in tests {
        let (before, before_pos, input, after_pos, expected) = test;

        let mut t = TextArea::from(before.iter().map(|s| s.to_string()));
        let (row, col) = before_pos;
        t.move_cursor(CursorMove::Jump(row, col));

        assert!(t.insert_str(input), "{:?}", test);
        assert_eq!(t.cursor(), after_pos, "{:?}", test);
        assert_eq!(t.lines(), expected, "{:?}", test);
    }
}
