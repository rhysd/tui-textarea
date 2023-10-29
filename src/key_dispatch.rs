use crate::cursor::CursorMove;
use crate::input::{Input, Key};
use crate::scroll::Scrolling;
use crate::TextArea;
pub(crate) fn input(ta: &mut TextArea<'_>, input: Input) -> bool {
    // Should we start selecting text, stop the current selection, or do nothing?
    // the end is handled after the ending keystroke
    let end_select = match (ta.select_start.is_some(), input.shift) {
        (true, true) => {
            // continue select
            false
        }
        (true, false) => {
            // end select
            true
        }
        (false, true) => {
            // start select
            ta.start_selection();
            false
        }
        (false, false) => {
            // ignore
            false
        }
    };
    let modified = match input {
        Input {
            key: Key::Char('m'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Char('\n' | '\r'),
            ctrl: false,
            alt: false,
            ..
        }
        | Input {
            key: Key::Enter, ..
        } => {
            ta.insert_newline();
            true
        }
        Input {
            key: Key::Char(c),
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.insert_char(c);
            true
        }
        Input {
            key: Key::Tab,
            ctrl: false,
            alt: false,
            ..
        } => ta.insert_tab(),
        Input {
            key: Key::Char('h'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Backspace,
            ctrl: false,
            alt: false,
            ..
        } => ta.delete_char(),
        Input {
            key: Key::Char('d'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Delete,
            ctrl: false,
            alt: false,
            ..
        } => ta.delete_next_char(),
        Input {
            key: Key::Char('k'),
            ctrl: true,
            alt: false,
            ..
        } => ta.delete_line_by_end(),
        Input {
            key: Key::Char('j'),
            ctrl: true,
            alt: false,
            ..
        } => ta.delete_line_by_head(),
        Input {
            key: Key::Char('w'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Char('h'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Backspace,
            ctrl: false,
            alt: true,
            ..
        } => ta.delete_word(),
        Input {
            key: Key::Delete,
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Char('d'),
            ctrl: false,
            alt: true,
            ..
        } => ta.delete_next_word(),
        Input {
            key: Key::Char('n' | 'N'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Down,
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::Down);
            false
        }
        Input {
            key: Key::Char('p' | 'P'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Up,
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::Up);
            false
        }
        Input {
            key: Key::Char('f' | 'F'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Right,
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::Forward);
            false
        }
        Input {
            key: Key::Char('b' | 'B'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::Left,
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::Back);
            false
        }
        Input {
            key: Key::Char('a' | 'A'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input { key: Key::Home, .. }
        | Input {
            key: Key::Left | Key::Char('b' | 'B'),
            ctrl: true,
            alt: true,
            ..
        } => {
            ta.move_cursor(CursorMove::Head);
            false
        }
        Input {
            key: Key::Char('e' | 'E'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input { key: Key::End, .. }
        | Input {
            key: Key::Right | Key::Char('f' | 'F'),
            ctrl: true,
            alt: true,
            ..
        } => {
            ta.move_cursor(CursorMove::End);
            false
        }
        Input {
            key: Key::Char('<'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Up | Key::Char('p' | 'P'),
            ctrl: true,
            alt: true,
            ..
        } => {
            ta.move_cursor(CursorMove::Top);
            false
        }
        Input {
            key: Key::Char('>'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Down | Key::Char('n' | 'N'),
            ctrl: true,
            alt: true,
            ..
        } => {
            ta.move_cursor(CursorMove::Bottom);
            false
        }
        Input {
            key: Key::Char('f' | 'F'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Right,
            ctrl: true,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::WordForward);
            false
        }
        Input {
            key: Key::Char('b' | 'B'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Left,
            ctrl: true,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::WordBack);
            false
        }
        Input {
            key: Key::Char(']'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Char('n' | 'N'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Down,
            ctrl: true,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::ParagraphForward);
            false
        }
        Input {
            key: Key::Char('['),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Char('p' | 'P'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::Up,
            ctrl: true,
            alt: false,
            ..
        } => {
            ta.move_cursor(CursorMove::ParagraphBack);
            false
        }
        Input {
            key: Key::Char('u'),
            ctrl: true,
            alt: false,
            ..
        } => ta.undo(),
        Input {
            key: Key::Char('r'),
            ctrl: true,
            alt: false,
            ..
        } => ta.redo(),
        Input {
            key: Key::Char('y'),
            ctrl: true,
            alt: false,
            ..
        } => ta.paste(),
        Input {
            key: Key::Char('c'),
            ctrl: true,
            alt: false,
            ..
        } => {
            ta.copy();
            false
        }
        Input {
            key: Key::Char('v'),
            ctrl: true,
            alt: false,
            ..
        }
        | Input {
            key: Key::PageDown, ..
        } => {
            ta.scroll(Scrolling::PageDown);
            false
        }
        Input {
            key: Key::Char('v'),
            ctrl: false,
            alt: true,
            ..
        }
        | Input {
            key: Key::PageUp, ..
        } => {
            ta.scroll(Scrolling::PageUp);
            false
        }
        Input {
            key: Key::MouseScrollDown,
            ..
        } => {
            ta.scroll((1, 0));
            false
        }
        Input {
            key: Key::MouseScrollUp,
            ..
        } => {
            ta.scroll((-1, 0));
            false
        }
        _ => false,
    };
    if end_select {
        ta.end_selection();
    };
    modified
}

pub(crate) fn input_without_shortcuts(ta: &mut TextArea<'_>, input: Input) -> bool {
    match input {
        Input {
            key: Key::Char(c),
            ctrl: false,
            alt: false,
            ..
        } => {
            ta.insert_char(c);
            true
        }
        Input {
            key: Key::Tab,
            ctrl: false,
            alt: false,
            ..
        } => ta.insert_tab(),
        Input {
            key: Key::Backspace,
            ..
        } => ta.delete_char(),
        Input {
            key: Key::Delete, ..
        } => ta.delete_next_char(),
        Input {
            key: Key::Enter, ..
        } => {
            ta.insert_newline();
            true
        }
        Input {
            key: Key::MouseScrollDown,
            ..
        } => {
            ta.scroll((1, 0));
            false
        }
        Input {
            key: Key::MouseScrollUp,
            ..
        } => {
            ta.scroll((-1, 0));
            false
        }
        _ => false,
    }
}
