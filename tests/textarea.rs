use std::cmp;
use std::fmt::Debug;
use tui_textarea::{CursorMove, TextArea};

fn assert_undo_redo<T: Debug>(
    before_pos: (usize, usize),
    before_buf: &[&str],
    after_buf: &[&str],
    t: &mut TextArea<'_>,
    context: T,
) {
    let after_pos = t.cursor();
    let modified = before_buf != after_buf;
    assert_eq!(t.cursor(), after_pos, "pos before undo: {context:?}");
    assert_eq!(t.undo(), modified, "undo modification: {context:?}");
    assert_eq!(t.lines(), before_buf, "buf after undo: {context:?}");
    assert_eq!(t.cursor(), before_pos, "pos after undo: {context:?}");
    assert_eq!(t.redo(), modified, "redo modification: {context:?}");
    assert_eq!(t.lines(), after_buf, "buf after redo: {context:?}");
    assert_eq!(t.cursor(), after_pos, "pos after redo: {context:?}");
}

fn assert_no_undo_redo<T: Debug>(t: &mut TextArea<'_>, context: T) {
    let pos = t.cursor();
    let buf: Vec<_> = t.lines().to_vec();
    assert!(!t.undo(), "undo modification: {context:?}");
    assert_eq!(t.lines(), &buf, "buf after undo: {context:?}");
    assert_eq!(t.cursor(), pos, "pos after undo: {context:?}");
    assert!(!t.redo(), "redo modification: {context:?}");
    assert_eq!(t.lines(), &buf, "buf after redo: {context:?}");
    assert_eq!(t.cursor(), pos, "pos after redo: {context:?}");
}

#[test]
fn test_insert_soft_tab() {
    for test in [
        ("", 0, "    ", 4),
        ("a", 1, "a   ", 3),
        ("abcd", 4, "abcd    ", 4),
        ("a", 0, "    a", 4),
        ("ab", 1, "a   b", 3),
        ("abcdefgh", 4, "abcd    efgh", 4),
        ("あ", 1, "あ  ", 2),
        ("🐶", 1, "🐶  ", 2),
        ("あ", 0, "    あ", 4),
        ("あい", 1, "あ  い", 2),
    ] {
        let (input, col, expected, width) = test;
        let mut t = TextArea::from([input.to_string()]);
        t.move_cursor(CursorMove::Jump(0, col));
        assert!(t.insert_tab(), "{test:?}");
        assert_eq!(t.lines(), [expected], "{test:?}");
        assert_eq!(t.cursor(), (0, col as usize + width), "{test:?}");
        assert_undo_redo((0, col as _), &[input], &[expected], &mut t, test);
    }
}

#[test]
fn test_insert_hard_tab() {
    let mut t = TextArea::default();
    t.set_hard_tab_indent(true);
    assert!(t.insert_tab());
    assert_eq!(t.cursor(), (0, 1));
    assert_undo_redo((0, 0), &[""], &["\t"], &mut t, "");

    let mut t = TextArea::default();
    t.set_hard_tab_indent(true);
    t.set_tab_length(0);
    t.insert_tab();
    assert!(!t.insert_tab());
    assert_eq!(t.lines(), [""]);
    assert_eq!(t.cursor(), (0, 0));
}

#[test]
fn test_insert_char() {
    let tests = [
        (0, 'x', &["xab"][..]),
        (1, 'x', &["axb"][..]),
        (2, 'x', &["abx"][..]),
        (1, 'あ', &["aあb"][..]),
        (1, '\n', &["a", "b"][..]),
    ];

    for test in tests {
        let (col, ch, want) = test;
        let mut t = TextArea::from(["ab"]);
        t.move_cursor(CursorMove::Jump(0, col));
        t.insert_char(ch);
        assert_eq!(t.lines(), want, "{test:?}");
        let pos = if ch == '\n' {
            (1, 0)
        } else {
            (0, col as usize + 1)
        };
        assert_eq!(t.cursor(), pos, "{test:?}");
        assert_undo_redo((0, col as _), &["ab"], want, &mut t, test);
    }
}

#[test]
fn test_insert_str_one_line() {
    for i in 0..="ab".len() {
        let mut t = TextArea::from(["ab"]);
        t.move_cursor(CursorMove::Jump(0, i as u16));
        assert!(t.insert_str("x"), "{i}");

        let mut want = "ab".to_string();
        want.insert(i, 'x');
        let want = want.as_str();
        assert_eq!(t.lines(), [want], "{i}");
        assert_eq!(t.cursor(), (0, i + 1));
        assert_undo_redo((0, i), &["ab"], &[want], &mut t, i);
    }

    let mut t = TextArea::default();
    assert!(t.insert_str("x"));
    assert_eq!(t.cursor(), (0, 1));
    assert_undo_redo((0, 0), &[""], &["x"], &mut t, "");
}

#[test]
fn test_insert_str_empty_line() {
    let mut t = TextArea::from(["ab"]);
    assert!(!t.insert_str(""));
    assert_eq!(t.lines(), ["ab"]);
    assert_eq!(t.cursor(), (0, 0));
    assert_no_undo_redo(&mut t, "");
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
        t.move_cursor(CursorMove::Jump(row as _, col as _));

        assert!(t.insert_str(input), "{test:?}");
        assert_eq!(t.cursor(), after_pos, "{test:?}");
        assert_eq!(t.lines(), expected, "{test:?}");

        assert_undo_redo(before_pos, before, expected, &mut t, test);
    }
}

#[test]
fn test_delete_str_nothing() {
    for i in 0..="ab".len() {
        let mut t = TextArea::from(["ab"]);
        assert!(!t.delete_str(0), "{i}");
        assert_eq!(t.cursor(), (0, 0));
    }
    let mut t = TextArea::default();
    assert!(!t.delete_str(0));
    assert_eq!(t.cursor(), (0, 0));
}

#[test]
fn test_delete_str_within_line() {
    for i in 0.."abc".len() {
        for j in 1..="abc".len() - i {
            let mut t = TextArea::from(["abc"]);
            t.move_cursor(CursorMove::Jump(0, i as _));
            assert!(t.delete_str(j), "at {i}, size={j}");

            let mut want = "abc".to_string();
            want.drain(i..i + j);
            let want = want.as_str();
            assert_eq!(t.lines(), [want], "at {i}, size={j}");
            assert_eq!(t.cursor(), (0, i));

            // delete_str deletes string as if moving cursor at the end of the deleted string
            assert_undo_redo((0, i + j), &["abc"], &[want], &mut t, (i, j));
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

        assert!(t.delete_str(chars), "{test:?}");
        assert_eq!(t.cursor(), (row, col), "{test:?}");
        assert_eq!(t.lines(), after, "{test:?}");
        assert_eq!(t.yank_text(), deleted, "{test:?}");

        let pos = t.cursor();
        assert!(t.undo(), "{test:?}");
        assert_eq!(t.lines(), before, "{test:?}");
        assert!(t.redo(), "{test:?}");
        assert_eq!(t.lines(), after, "{test:?}");
        assert_eq!(t.cursor(), pos, "{test:?}");
    }
}

#[test]
fn test_copy_single_line() {
    for i in 0..="abc".len() {
        for j in i.."abc".len() {
            let mut t = TextArea::from(["abc"]);

            t.move_cursor(CursorMove::Jump(0, i as u16));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(0, j as u16));
            t.copy();

            assert_eq!(t.yank_text(), &"abc"[i..j], "from {i} to {j}");
            assert_eq!(t.lines(), ["abc"], "from {i} to {j}");

            assert_no_undo_redo(&mut t, (i, j));
        }
    }
}

#[test]
fn test_cut_single_line() {
    for i in 0.."abc".len() {
        for j in i + 1.."abc".len() {
            let mut t = TextArea::from(["abc"]);

            t.move_cursor(CursorMove::Jump(0, i as u16));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(0, j as u16));
            t.cut();

            assert_eq!(t.yank_text(), &"abc"[i..j], "from {i} to {j}");

            let mut after = "abc".to_string();
            after.replace_range(i..j, "");
            let after = after.as_str();
            assert_eq!(t.lines(), [after], "from {i} to {j}");
            assert_eq!(t.cursor(), (0, i));
            assert_undo_redo((0, j), &["abc"], &[after], &mut t, (i, j));

            t.paste();
            assert_eq!(t.lines(), ["abc"], "from {i} to {j}");
            assert_undo_redo((0, i), &[after], &["abc"], &mut t, (i, j));
        }
    }
}

#[test]
fn test_copy_cut_empty() {
    for row in 0..=2 {
        for col in 0..=2 {
            let check = |f: fn(&mut TextArea<'_>)| {
                let mut t = TextArea::from(["ab", "cd", "ef"]);
                t.move_cursor(CursorMove::Jump(row, col));
                t.start_selection();
                t.move_cursor(CursorMove::Jump(row, col));
                f(&mut t);
                assert!(!t.is_selecting());
                assert_eq!(t.cursor(), (row as _, col as _));
                assert_eq!(t.lines(), ["ab", "cd", "ef"]);
                assert_no_undo_redo(&mut t, "");
            };

            check(|t| {
                assert!(!t.cut());
            });
            check(|t| t.copy());
        }
    }
}

#[test]
fn test_copy_cut_paste_multi_lines() {
    #[rustfmt::skip]
    let tests = [
        (
            // Initial text
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            // Start position of selection
            (0, 0),
            // End position of selection
            (1, 0),
            // Expected yanked text
            "ab\n",
            // Text buffer after cut
            &[
                "cd",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            (1, 1),
            "ab\nc",
            &[
                "d",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            (1, 2),
            "ab\ncd",
            &[
                "",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            (2, 0),
            "ab\ncd\n",
            &[
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            (2, 1),
            "ab\ncd\ne",
            &[
                "f",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 0),
            (2, 2),
            "ab\ncd\nef",
            &[
                "",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 1),
            (1, 1),
            "b\nc",
            &[
                "ad",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 2),
            (1, 1),
            "\nc",
            &[
                "abd",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (1, 0),
            (2, 1),
            "cd\ne",
            &[
                "ab",
                "f",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 2),
            (1, 0),
            "\n",
            &[
                "abcd",
                "ef",
            ][..]
        ),
        (
            &[
                "ab",
                "cd",
                "ef",
            ][..],
            (0, 2),
            (2, 0),
            "\ncd\n",
            &[
                "abef",
            ][..]
        ),
        // Multi-byte characters
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 0),
            (2, 2),
            "あい\nうえ\nおか",
            &[
                "",
            ][..]
        ),
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 1),
            (2, 1),
            "い\nうえ\nお",
            &[
                "あか",
            ][..]
        ),
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 2),
            (2, 0),
            "\nうえ\n",
            &[
                "あいおか",
            ][..]
        ),
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 2),
            (1, 2),
            "\nうえ",
            &[
                "あい",
                "おか",
            ][..]
        ),
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 2),
            (1, 1),
            "\nう",
            &[
                "あいえ",
                "おか",
            ][..]
        ),
        (
            &[
                "あい",
                "うえ",
                "おか",
            ][..],
            (0, 2),
            (1, 0),
            "\n",
            &[
                "あいうえ",
                "おか",
            ][..]
        ),
    ];

    for test in tests {
        let (init_text, (srow, scol), (erow, ecol), yanked, after_cut) = test;

        {
            let mut t = TextArea::from(init_text.iter().map(|s| s.to_string()));
            t.move_cursor(CursorMove::Jump(srow as _, scol as _));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(erow as _, ecol as _));
            t.copy();

            assert_eq!(t.cursor(), (erow, ecol), "{test:?}");
            assert_eq!(t.yank_text(), yanked, "{test:?}");
            assert_eq!(t.lines(), init_text, "{test:?}");
            assert_no_undo_redo(&mut t, test);
        }

        {
            let mut t = TextArea::from(init_text.iter().map(|s| s.to_string()));
            t.move_cursor(CursorMove::Jump(srow as _, scol as _));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(erow as _, ecol as _));
            t.cut();

            assert_eq!(t.cursor(), (srow, scol), "{test:?}");
            assert_eq!(t.yank_text(), yanked, "{test:?}");
            assert_eq!(t.lines(), after_cut, "{test:?}");
            assert_undo_redo((erow, ecol), init_text, after_cut, &mut t, test);

            t.paste();
            assert_eq!(t.lines(), init_text, "{test:?}");
            assert_undo_redo((srow, scol), after_cut, init_text, &mut t, test);
        }

        // Reverse positions
        {
            let mut t = TextArea::from(init_text.iter().map(|s| s.to_string()));
            t.move_cursor(CursorMove::Jump(erow as _, ecol as _));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(srow as _, scol as _));
            t.copy();

            assert_eq!(t.cursor(), (srow, scol), "{test:?}");
            assert_eq!(t.yank_text(), yanked, "{test:?}");
            assert_eq!(t.lines(), init_text, "{test:?}");
            assert_no_undo_redo(&mut t, test);
        }

        {
            let mut t = TextArea::from(init_text.iter().map(|s| s.to_string()));
            t.move_cursor(CursorMove::Jump(erow as _, ecol as _));
            t.start_selection();
            t.move_cursor(CursorMove::Jump(srow as _, scol as _));
            t.cut();

            assert_eq!(t.cursor(), (srow, scol), "{test:?}");
            assert_eq!(t.yank_text(), yanked, "{test:?}");
            assert_eq!(t.lines(), after_cut, "{test:?}");
            assert_undo_redo((erow, ecol), init_text, after_cut, &mut t, test);

            t.paste();
            assert_eq!(t.lines(), init_text, "{test:?}");
            assert_undo_redo((srow, scol), after_cut, init_text, &mut t, test);
        }
    }
}

#[test]
fn test_delete_selection_on_delete_operations() {
    macro_rules! test_case {
        ($name:ident($($args:expr),*)) => {
            (
                stringify!($name),
                (|t| t.$name($($args),*)) as fn(&mut TextArea) -> bool,
            )
        };
    }

    let tests = [
        test_case!(delete_char()),
        test_case!(delete_next_char()),
        test_case!(delete_line_by_end()),
        test_case!(delete_line_by_head()),
        test_case!(delete_word()),
        test_case!(delete_next_word()),
        test_case!(delete_str(3)),
    ];

    for (n, f) in tests {
        let mut t = TextArea::from(["ab", "cd", "ef"]);
        t.move_cursor(CursorMove::Jump(0, 1));
        t.start_selection();
        t.move_cursor(CursorMove::Jump(2, 1));

        let modified = f(&mut t);
        assert!(modified, "{n}");
        assert_eq!(t.lines(), ["af"], "{n}");
        assert_eq!(t.cursor(), (0, 1), "{n}");

        assert_undo_redo((2, 1), &["ab", "cd", "ef"], &["af"], &mut t, n);
    }
}

#[test]
fn test_delete_selection_on_delete_edge_cases() {
    macro_rules! test_case {
        ($name:ident($($args:expr),*), $pos:expr) => {
            (
                stringify!($name),
                (|t| t.$name($($args),*)) as fn(&mut TextArea) -> bool,
                $pos,
            )
        };
    }

    // When deleting nothing and deleting newline
    let tests = [
        test_case!(delete_char(), (0, 0)),
        test_case!(delete_char(), (1, 0)),
        test_case!(delete_next_char(), (2, 2)),
        test_case!(delete_next_char(), (1, 2)),
        test_case!(delete_line_by_end(), (0, 2)),
        test_case!(delete_line_by_end(), (2, 2)),
        test_case!(delete_line_by_head(), (0, 0)),
        test_case!(delete_line_by_head(), (1, 0)),
        test_case!(delete_word(), (0, 0)),
        test_case!(delete_word(), (1, 0)),
        test_case!(delete_next_word(), (2, 2)),
        test_case!(delete_next_word(), (1, 2)),
        test_case!(delete_str(0), (0, 0)),
        test_case!(delete_str(100), (2, 2)),
    ];

    for (n, f, pos) in tests {
        let mut t = TextArea::from(["ab", "cd", "ef"]);
        t.move_cursor(CursorMove::Jump(1, 1));
        t.start_selection();
        t.move_cursor(CursorMove::Jump(pos.0 as _, pos.1 as _));

        assert!(f(&mut t), "{n}, {pos:?}");
        assert_eq!(t.cursor(), cmp::min(pos, (1, 1)), "{n}, {pos:?}");

        t.undo();
        assert_eq!(t.lines(), ["ab", "cd", "ef"], "{n}, {pos:?}");
    }
}

#[test]
fn test_delete_selection_before_insert() {
    macro_rules! test_case {
        ($name:ident($($args:expr),*), $want:expr) => {
            (
                stringify!($name),
                (|t| {
                    t.$name($($args),*);
                }) as fn(&mut TextArea),
                &$want as &[_],
            )
        };
    }

    let tests = [
        test_case!(insert_newline(), ["a", "f"]),
        test_case!(insert_char('x'), ["axf"]),
        test_case!(insert_tab(), ["a   f"]), // Default tab is 4 spaces
        test_case!(insert_str("xyz"), ["axyzf"]),
    ];

    for (n, f, after) in tests {
        let mut t = TextArea::from(["ab", "cd", "ef"]);
        t.move_cursor(CursorMove::Jump(0, 1));
        t.start_selection();
        t.move_cursor(CursorMove::Jump(2, 1));

        f(&mut t);
        assert_eq!(t.lines(), after, "{n}");

        // XXX: Deleting selection and inserting text are separate undo units for now
        t.undo();
        t.undo();
        assert_eq!(t.lines(), ["ab", "cd", "ef"], "{n}");
    }
}

#[test]
fn test_undo_redo_stop_selection() {
    fn check(t: &mut TextArea, f: fn(&mut TextArea) -> bool) {
        t.move_cursor(CursorMove::Jump(0, 0));
        t.start_selection();
        t.move_cursor(CursorMove::Jump(0, 1));
        assert!(t.is_selecting());
        assert!(f(t));
        assert!(!t.is_selecting());
    }

    let mut t = TextArea::default();
    t.insert_char('a');

    check(&mut t, |t| t.undo());
    assert_eq!(t.lines(), [""]);
    check(&mut t, |t| t.redo());
    assert_eq!(t.lines(), ["a"]);
}

#[test]
fn test_set_yank_paste_text() {
    let tests = [
        ("", &[""][..], (0, 0)),
        ("abc", &["abc"][..], (0, 3)),
        ("abc\ndef", &["abc", "def"][..], (1, 3)),
        ("\n\n", &["", "", ""][..], (2, 0)),
    ];

    for test in tests {
        let (text, want, pos) = test;
        let mut t = TextArea::default();
        t.set_yank_text(text);
        t.paste();
        assert_eq!(t.lines(), want, "{test:?}");
        assert_eq!(t.yank_text(), text, "{test:?}");
        assert_eq!(t.cursor(), pos, "{test:?}");
        assert_undo_redo((0, 0), &[""], want, &mut t, test);
    }
}

#[test]
fn test_select_all() {
    let mut t = TextArea::from(["aaa", "bbb", "ccc"]);
    t.select_all();
    assert!(t.is_selecting());
    assert_eq!(t.cursor(), (2, 3));
    t.cut();
    assert_eq!(t.lines(), [""]);
    assert_eq!(t.yank_text(), "aaa\nbbb\nccc");
    assert_undo_redo((2, 3), &["aaa", "bbb", "ccc"], &[""], &mut t, "");
}

struct DeleteTester(&'static [&'static str], fn(&mut TextArea) -> bool);
impl DeleteTester {
    fn test(&self, before: (usize, usize), after: (usize, usize, &[&str], &str)) {
        let Self(buf_before, op) = *self;
        let (row, col) = before;

        let mut t = TextArea::from(buf_before.iter().map(|s| s.to_string()));
        t.move_cursor(CursorMove::Jump(row as _, col as _));
        let modified = op(&mut t);

        let (row, col, buf_after, yank) = after;
        assert_eq!(t.lines(), buf_after);
        assert_eq!(t.cursor(), (row, col));
        assert_eq!(modified, buf_before != buf_after);
        assert_eq!(t.yank_text(), yank);

        if modified {
            t.undo();
            assert_eq!(t.lines(), buf_before);
            t.redo();
            assert_eq!(t.lines(), buf_after);
        } else {
            assert_no_undo_redo(&mut t, "");
        }
    }
}

#[test]
fn test_delete_newline() {
    let t = DeleteTester(&["a", "b", "c"], |t| t.delete_newline());
    t.test((0, 0), (0, 0, t.0, ""));
    t.test((1, 0), (0, 1, &["ab", "c"], ""));
    t.test((2, 0), (1, 1, &["a", "bc"], ""));
}

#[test]
fn test_delete_char() {
    let t = DeleteTester(&["ab", "c"], |t| t.delete_char());
    t.test((0, 0), (0, 0, t.0, ""));
    t.test((0, 1), (0, 0, &["b", "c"], ""));
    t.test((0, 2), (0, 1, &["a", "c"], ""));
    t.test((1, 0), (0, 2, &["abc"], ""));
}

#[test]
fn test_delete_next_char() {
    let t = DeleteTester(&["ab", "c"], |t| t.delete_next_char());
    t.test((0, 0), (0, 0, &["b", "c"], ""));
    t.test((0, 1), (0, 1, &["a", "c"], ""));
    t.test((0, 2), (0, 2, &["abc"], ""));
    t.test((1, 1), (1, 1, t.0, ""));
}

#[test]
fn test_delete_line_by_end() {
    let t = DeleteTester(&["aaa bbb", "d"], |t| t.delete_line_by_end());
    t.test((0, 0), (0, 0, &["", "d"], "aaa bbb"));
    t.test((0, 3), (0, 3, &["aaa", "d"], " bbb"));
    t.test((0, 6), (0, 6, &["aaa bb", "d"], "b"));
    t.test((0, 7), (0, 7, &["aaa bbbd"], "")); // Newline is not yanked
    t.test((1, 1), (1, 1, t.0, ""));
}

#[test]
fn test_delete_line_by_head() {
    let t = DeleteTester(&["aaa bbb", "d"], |t| t.delete_line_by_head());
    t.test((0, 0), (0, 0, t.0, ""));
    t.test((0, 3), (0, 0, &[" bbb", "d"], "aaa"));
    t.test((0, 7), (0, 0, &["", "d"], "aaa bbb"));
    t.test((1, 0), (0, 7, &["aaa bbbd"], "")); // Newline is not yanked
}

#[test]
fn test_delete_word() {
    let t = DeleteTester(&["word  ことば 🐶", " x"], |t| t.delete_word());
    t.test((0, 0), (0, 0, t.0, ""));
    t.test((0, 2), (0, 0, &["rd  ことば 🐶", " x"], "wo"));
    t.test((0, 4), (0, 0, &["  ことば 🐶", " x"], "word"));
    t.test((0, 5), (0, 0, &[" ことば 🐶", " x"], "word "));
    t.test((0, 6), (0, 0, &["ことば 🐶", " x"], "word  "));
    t.test((0, 7), (0, 6, &["word  とば 🐶", " x"], "こ"));
    t.test((0, 9), (0, 6, &["word   🐶", " x"], "ことば"));
    t.test((0, 10), (0, 6, &["word  🐶", " x"], "ことば "));
    t.test((0, 11), (0, 10, &["word  ことば ", " x"], "🐶"));
    t.test((1, 0), (0, 11, &["word  ことば 🐶 x"], ""));
    t.test((1, 1), (1, 0, &["word  ことば 🐶", "x"], " "));
    t.test((1, 2), (1, 1, &["word  ことば 🐶", " "], "x"));
}

#[test]
fn test_delete_next_word() {
    let t = DeleteTester(&["word  ことば 🐶", " x"], |t| t.delete_next_word());
    t.test((0, 0), (0, 0, &["  ことば 🐶", " x"], "word"));
    t.test((0, 2), (0, 2, &["wo  ことば 🐶", " x"], "rd"));
    t.test((0, 4), (0, 4, &["word 🐶", " x"], "  ことば"));
    t.test((0, 5), (0, 5, &["word  🐶", " x"], " ことば"));
    t.test((0, 6), (0, 6, &["word   🐶", " x"], "ことば"));
    t.test((0, 9), (0, 9, &["word  ことば", " x"], " 🐶"));
    t.test((0, 10), (0, 10, &["word  ことば ", " x"], "🐶"));
    t.test((0, 11), (0, 11, &["word  ことば 🐶 x"], ""));
    t.test((1, 0), (1, 0, &["word  ことば 🐶", ""], " x"));
    t.test((1, 2), (1, 2, t.0, ""));
}
