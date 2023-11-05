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

//
//            selection tests
//

use tui_textarea::{Input, Key};

fn make_input(k: (Key, bool, bool, bool)) -> Input {
    Input {
        key: k.0,
        ctrl: k.1,
        alt: k.2,
        shift: k.3,
    }
}
fn input_key(textarea: &mut TextArea, k: (Key, bool, bool, bool)) {
    textarea.input(make_input(k));
}
fn clear(ta: &mut TextArea) {
    start(ta);
    ta.delete_str(usize::MAX);
}

fn start(ta: &mut TextArea) {
    input_key(ta, (Key::Up, true, true, false));
    input_key(ta, (Key::Home, false, false, false));
}

#[test]
fn select_copy() {
    let mut textarea = TextArea::default();
    for test in [
        // plain ascii
        (
            "hello world",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
                (Key::Right, false, false, true),
            ],
            "he",
        ),
        // plain ascii backwards
        (
            "hello world",
            vec![
                (Key::End, false, false, false),
                (Key::Left, false, false, true),
                (Key::Left, false, false, true),
            ],
            "ld",
        ),
        // utf8
        (
            "あい",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
            ],
            "あ",
        ),
        // multi line - all
        (
            "hello\nworld",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
            ],
            "hello\nworld",
        ),
        // multi line - some
        (
            "hello\nworld",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Right, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
                (Key::Left, false, false, true),
            ],
            "ello\nworl",
        ),
        // multi - line utf8
        (
            "あい\nうえ",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
            ],
            "あい\nうえ",
        ),
        // multi-line utf8 - some
        (
            "あい\nうえ",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Right, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
                (Key::Left, false, false, true),
            ],
            "い\nう",
        ),
    ] {
        clear(&mut textarea);
        textarea.insert_str(test.0);
        for k in test.1 {
            input_key(&mut textarea, k);
        }
        textarea.copy();
        assert_eq!(textarea.yank_text(), test.2);
    }
}
#[test]
fn select_cut() {
    let mut textarea = TextArea::default();
    for test in [
        // plain ascii
        (
            "hello world",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
                (Key::Right, false, false, true),
            ],
            "he",
            "llo world",
        ),
        // plain ascii backwards
        (
            "hello world",
            vec![
                (Key::End, false, false, false),
                (Key::Left, false, false, true),
                (Key::Left, false, false, true),
            ],
            "ld",
            "hello wor",
        ),
        // utf8
        (
            "あい",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
            ],
            "あ",
            "い",
        ),
        // multi line - all
        (
            "hello\nworld",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
            ],
            "hello\nworld",
            "",
        ),
        // multi line - some
        (
            "hello\nworld",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Right, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
                (Key::Left, false, false, true),
            ],
            "ello\nworl",
            "hd",
        ),
        // multi - line utf8
        (
            "あい\nうえ",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
            ],
            "あい\nうえ",
            "",
        ),
        // multi-line utf8 - some
        (
            "あい\nうえ",
            vec![
                (Key::Char('P'), true, true, false),
                (Key::Home, false, false, false),
                (Key::Right, false, false, false),
                (Key::Char('N'), true, true, true),
                (Key::End, false, false, true),
                (Key::Left, false, false, true),
            ],
            "い\nう",
            "あえ",
        ),
    ] {
        clear(&mut textarea);
        textarea.insert_str(test.0);
        for k in test.1 {
            input_key(&mut textarea, k);
        }
        textarea.cut();
        assert_eq!(textarea.yank_text(), test.2);
        assert_eq!(textarea.lines().join("\n"), test.3);
    }
}

#[test]
fn select_paste() {
    let mut textarea = TextArea::default();
    for test in [
        // plain ascii
        (
            "hello world",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
                (Key::Right, false, false, true),
                (Key::Char('c'), true, false, false),
                (Key::End, false, false, false),
                (Key::Char('y'), true, false, false),
            ],
            "he",
            "hello worldhe",
        ),
        (
            "hello world",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, true),
                (Key::Right, false, false, true),
                (Key::Char('c'), true, false, false),
                (Key::End, false, false, false),
                (Key::Left, false, false, true),
                (Key::Left, false, false, true),
                (Key::Char('y'), true, false, false),
            ],
            "he",
            "hello worhe",
        ),
        (
            "hello\nworld",
            vec![
                (Key::Home, false, false, false),
                (Key::Right, false, false, false),
                (Key::Right, false, false, false),
                (Key::Down, false, false, true),
                (Key::Char('c'), true, false, false),
                (Key::End, false, false, false),
                (Key::Enter, false, false, false),
                (Key::Char('y'), true, false, false),
            ],
            "llo\nwo",
            "hello\nworld\nllo\nwo",
        ),
    ] {
        clear(&mut textarea);
        textarea.insert_str(test.0);
        start(&mut textarea);
        for k in test.1 {
            input_key(&mut textarea, k);
        }
        assert_eq!(textarea.yank_text(), test.2);
        assert_eq!(textarea.lines().join("\n"), test.3);
    }
}
#[test]
fn select_all_keys() {
    let mut textarea = TextArea::default();
    for test in [
        (
            // enter key erases selection
            "hello world",
            vec![
                (Key::End, false, false, true),
                (Key::Enter, false, false, true),
            ],
            "",
            "\n",
        ),
        (
            // word forward select
            "hello world",
            vec![
                (Key::Char('f'), false, true, true),
                (Key::Char('c'), true, false, false),
            ],
            "hello ",
            "hello world",
        ),
        (
            // word back select
            "hello world",
            vec![
                (Key::End, false, false, false),
                (Key::Char('b'), false, true, true),
                (Key::Char('c'), true, false, false),
            ],
            "world",
            "hello world",
        ),
        (
            // para forward select
            "hello\nworld\n\nhow\nare\nyou",
            vec![
                (Key::Char('n'), false, true, true),
                (Key::Char('x'), true, false, true),
            ],
            "hello\nworld\n\n",
            "how\nare\nyou",
        ),
        (
            // para back select
            "hello\nworld\n\nhow\nare\nyou",
            vec![
                // goto end
                (Key::Char('n'), true, true, false),
                (Key::End, false, false, false),
                (Key::Char('p'), false, true, true),
                (Key::Char('x'), true, false, true),
            ],
            "\nare\nyou",
            "hello\nworld\n\nhow",
        ),
        (
            // undo
            "hello\nworld\n\nhow\nare\nyou",
            vec![
                // goto end
                (Key::Char('n'), true, true, false),
                (Key::End, false, false, false),
                (Key::Char('p'), false, true, true),
                (Key::Char('x'), true, false, false),
                (Key::Char('u'), true, false, false),
            ],
            "\nare\nyou",
            "hello\nworld\n\nhow\nare\nyou",
        ),
    ] {
        clear(&mut textarea);
        textarea.insert_str(test.0);
        start(&mut textarea);
        for k in test.1 {
            input_key(&mut textarea, k);
        }

        assert_eq!(textarea.yank_text(), test.2);
        assert_eq!(textarea.lines().join("\n"), test.3);
    }
}
