use tui_textarea::{CursorMove, Input, Key, TextArea};

#[test]
fn empty_selection_delete_does_not_invent_undo() {
    let mut area = TextArea::default();
    area.select_all();
    assert!(!area.delete_str(usize::MAX));
    assert!(!area.input(Input {
        key: Key::Char('u'),
        ctrl: true,
        ..Input::default()
    }));
    assert_eq!(area.lines(), [""]);
    assert_eq!(area.cursor(), (0, 0));
}

#[test]
fn oversized_delete_restores_existing_last_line() {
    for lines in [vec!["abc"], vec!["abc", "界🙂"]] {
        let mut area = TextArea::from(lines.clone());
        area.move_cursor(CursorMove::Head);
        assert!(area.delete_str(usize::MAX));
        assert!(area.input(Input {
            key: Key::Char('u'),
            ctrl: true,
            ..Input::default()
        }));
        assert_eq!(area.lines(), lines);
        let (row, col) = area.cursor();
        assert!(row < area.lines().len());
        assert!(col <= area.lines()[row].chars().count());
    }
}

#[test]
fn deleting_past_empty_end_preserves_redo() {
    let mut area = TextArea::default();
    area.insert_str("界");
    assert!(area.undo());
    area.select_all();
    assert!(!area.delete_str(usize::MAX));
    assert!(area.redo());
    assert_eq!(area.lines(), ["界"]);
    assert_eq!(area.cursor(), (0, 1));
}

#[test]
fn oversized_delete_from_final_cursor_is_a_noop() {
    let mut area = TextArea::from(["one", "界🙂"]);
    area.move_cursor(CursorMove::Bottom);
    area.move_cursor(CursorMove::End);
    assert!(!area.delete_str(usize::MAX));
    assert!(!area.undo());
    assert_eq!(area.lines(), ["one", "界🙂"]);
    assert_eq!(area.cursor(), (1, 2));
}
