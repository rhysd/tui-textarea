use tui_textarea::{CursorMove, TextArea};

const BOTTOM_RIGHT: CursorMove = CursorMove::Jump(u16::MAX, u16::MAX);

#[test]
fn test_empty_textarea() {
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
fn test_forward() {
    let mut t = TextArea::from(["abc", "def"]);

    for pos in [
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 0),
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 3),
    ] {
        t.move_cursor(CursorMove::Forward);
        assert_eq!(t.cursor(), pos);
    }
}

#[test]
fn test_back() {
    let mut t = TextArea::from(["abc", "def"]);
    t.move_cursor(BOTTOM_RIGHT);

    for pos in [
        (1, 2),
        (1, 1),
        (1, 0),
        (0, 3),
        (0, 2),
        (0, 1),
        (0, 0),
        (0, 0),
    ] {
        t.move_cursor(CursorMove::Back);
        assert_eq!(t.cursor(), pos);
    }
}
