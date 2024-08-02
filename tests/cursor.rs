use tui_textarea::{CursorMove, TextArea};

const BOTTOM_RIGHT: CursorMove = CursorMove::Jump(u16::MAX, u16::MAX);

#[test]
fn empty_textarea() {
    use CursorMove::*;

    let mut t = TextArea::default();
    for m in [
        Forward,
        Back,
        Up,
        Down,
        Head,
        End,
        Top,
        Bottom,
        WordForward,
        WordEnd,
        WordBack,
        ParagraphForward,
        ParagraphBack,
        Jump(0, 0),
        Jump(u16::MAX, u16::MAX),
    ] {
        t.move_cursor(m);
        assert_eq!(t.cursor(), (0, 0), "{:?}", m);
    }
}

#[test]
fn forward() {
    for (text, positions) in [
        (
            ["abc", "def"],
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 1),
                (1, 2),
                (1, 3),
                (1, 3),
            ],
        ),
        (
            ["あいう", "🐶🐱👪"],
            [
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 1),
                (1, 2),
                (1, 3),
                (1, 3),
            ],
        ),
    ] {
        let mut t = TextArea::from(text);

        for pos in positions {
            t.move_cursor(CursorMove::Forward);
            assert_eq!(t.cursor(), pos, "{:?}", t.lines());
        }
    }
}

#[test]
fn back() {
    for (text, positions) in [
        (
            ["abc", "def"],
            [
                (1, 2),
                (1, 1),
                (1, 0),
                (0, 3),
                (0, 2),
                (0, 1),
                (0, 0),
                (0, 0),
            ],
        ),
        (
            ["あいう", "🐶🐱👪"],
            [
                (1, 2),
                (1, 1),
                (1, 0),
                (0, 3),
                (0, 2),
                (0, 1),
                (0, 0),
                (0, 0),
            ],
        ),
    ] {
        let mut t = TextArea::from(text);
        t.move_cursor(BOTTOM_RIGHT);

        for pos in positions {
            t.move_cursor(CursorMove::Back);
            assert_eq!(t.cursor(), pos, "{:?}", t.lines());
        }
    }
}

#[test]
fn up() {
    for text in [
        ["abc", "def", "ghi"],
        ["あいう", "🐶🐱🐰", "👪🤟🏿👩🏻‍❤️‍💋‍👨🏾"],
    ] {
        let mut t = TextArea::from(text);

        for col in 0..=3 {
            let mut row = 2;

            t.move_cursor(CursorMove::Jump(2, col as u16));
            assert_eq!(t.cursor(), (row, col), "{:?}", t.lines());

            while row > 0 {
                t.move_cursor(CursorMove::Up);
                row -= 1;
                assert_eq!(t.cursor(), (row, col), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn up_trim() {
    for text in [["", "a", "bcd", "efgh"], ["", "👪", "🐶!🐱", "あ?い!"]] {
        let mut t = TextArea::from(text);
        t.move_cursor(CursorMove::Jump(3, 3));

        for expected in [(2, 3), (1, 1), (0, 0)] {
            t.move_cursor(CursorMove::Up);
            assert_eq!(t.cursor(), expected, "{:?}", t.lines());
        }
    }
}

#[test]
fn down() {
    for text in [
        ["abc", "def", "ghi"],
        ["あいう", "🐶🐱🐰", "👪🤟🏿👩🏻‍❤️‍💋‍👨🏾"],
    ] {
        let mut t = TextArea::from(text);

        for col in 0..=3 {
            let mut row = 0;

            t.move_cursor(CursorMove::Jump(0, col as u16));
            assert_eq!(t.cursor(), (row, col), "{:?}", t.lines());

            while row < 2 {
                t.move_cursor(CursorMove::Down);
                row += 1;
                assert_eq!(t.cursor(), (row, col), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn down_trim() {
    for text in [["abcd", "efg", "h", ""], ["あ?い!", "🐶!🐱", "👪", ""]] {
        let mut t = TextArea::from(text);
        t.move_cursor(CursorMove::Jump(0, 3));

        for expected in [(1, 3), (2, 1), (3, 0)] {
            t.move_cursor(CursorMove::Down);
            assert_eq!(t.cursor(), expected, "{:?}", t.lines());
        }
    }
}

#[test]
fn head() {
    for text in [["efg", "h", ""], ["あいう", "👪", ""]] {
        let mut t = TextArea::from(text);
        for row in 0..t.lines().len() {
            let len = t.lines()[row].len();
            for col in [0, len / 2, len] {
                t.move_cursor(CursorMove::Jump(row as u16, col as u16));
                t.move_cursor(CursorMove::Head);
                assert_eq!(t.cursor(), (row, 0), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn end() {
    for text in [["efg", "h", ""], ["あいう", "👪", ""]] {
        let mut t = TextArea::from(text);
        for row in 0..t.lines().len() {
            let len = match row {
                0 => 3,
                1 => 1,
                2 => 0,
                _ => unreachable!(),
            };
            for col in [0, len / 2, len] {
                t.move_cursor(CursorMove::Jump(row as u16, col as u16));
                t.move_cursor(CursorMove::End);
                assert_eq!(t.cursor(), (row, len), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn top() {
    for text in [
        ["abc", "def", "ghi"],
        ["あいう", "🐶🐱🐰", "👪🤟🏿👩🏻‍❤️‍💋‍👨🏾"],
    ] {
        let mut t = TextArea::from(text);
        for row in 0..=2 {
            for col in 0..=3 {
                t.move_cursor(CursorMove::Jump(row, col));
                t.move_cursor(CursorMove::Top);
                assert_eq!(t.cursor(), (0, col as usize), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn top_trim() {
    for lines in [
        &["a", "bc"][..],
        &["あ", "🐶🐱"][..],
        &["a", "bcd", "ef"][..],
        &["", "犬"][..],
    ] {
        let mut t: TextArea = lines.iter().cloned().collect();
        t.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
        t.move_cursor(CursorMove::Top);
        let col = t.lines()[0].chars().count();
        assert_eq!(t.cursor(), (0, col), "{:?}", t.lines());
    }
}

#[test]
fn bottom() {
    for text in [
        ["abc", "def", "ghi"],
        ["あいう", "🐶🐱🐰", "👪🤟🏿👩🏻‍❤️‍💋‍👨🏾"],
    ] {
        let mut t = TextArea::from(text);
        for row in 0..=2 {
            for col in 0..=3 {
                t.move_cursor(CursorMove::Jump(row, col));
                t.move_cursor(CursorMove::Bottom);
                assert_eq!(t.cursor(), (2, col as usize), "{:?}", t.lines());
            }
        }
    }
}

#[test]
fn bottom_trim() {
    for lines in [
        &["bc", "a"][..],
        &["🐶🐱", "🐰"][..],
        &["ef", "bcd", "a"][..],
        &["犬", ""][..],
    ] {
        let mut t: TextArea = lines.iter().cloned().collect();
        t.move_cursor(CursorMove::Jump(0, u16::MAX));
        t.move_cursor(CursorMove::Bottom);
        let col = t.lines().last().unwrap().chars().count();
        assert_eq!(t.cursor(), (t.lines().len() - 1, col), "{:?}", t.lines());
    }
}

#[test]
fn word_end() {
    for (lines, positions) in [
        (
            &[
                "aaa !!! bbb", // Consecutive punctuations are a word
            ][..],
            &[(0, 2), (0, 6), (0, 10)][..],
        ),
        (
            &[
                "aaa!!!bbb", // Word boundaries without spaces
            ][..],
            &[(0, 2), (0, 5), (0, 8)][..],
        ),
        (
            &[
                "aaa", "", "", "bbb", // Go across multiple empty lines (regression of #75)
            ][..],
            &[(0, 2), (3, 2)][..],
        ),
        (
            &[
                "aaa", "   ", "   ", "bbb", // Go across multiple blank lines
            ][..],
            &[(0, 2), (3, 2)][..],
        ),
        (
            &[
                "   aaa", "   bbb", // Ignore the spaces at the head of line
            ][..],
            &[(0, 5), (1, 5)][..],
        ),
        (
            &[
                "aaa   ", "bbb   ", // Ignore the spaces at the end of line
            ][..],
            &[(0, 2), (1, 2), (1, 6)][..],
        ),
        (
            &[
                "a aa", "b!!!", // Accept the head of line (regression of #75)
            ][..],
            &[(0, 3), (1, 0), (1, 3)][..],
        ),
    ] {
        let mut t: TextArea = lines.iter().cloned().collect();
        for pos in positions {
            t.move_cursor(CursorMove::WordEnd);
            assert_eq!(t.cursor(), *pos, "{:?}", t.lines());
        }
    }
}
