use super::{Input, Key};
#[cfg(target_os = "windows")]
use crate::crossterm::event::KeyEventKind;
use crate::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

impl From<Event> for Input {
    /// Convert [`crossterm::event::Event`] to [`Input`].
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => Self::from(key),
            Event::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

impl From<KeyEvent> for Input {
    /// Convert [`crossterm::event::KeyEvent`] to [`Input`].
    fn from(key: KeyEvent) -> Self {
        #[cfg(target_os = "windows")]
        if key.kind == KeyEventKind::Release {
            // On Windows, handle only button press events (#14)
            return Self::default();
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let key = match key.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Tab => Key::Tab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Esc => Key::Esc,
            KeyCode::F(x) => Key::F(x),
            _ => Key::Null,
        };

        Self { key, ctrl, alt }
    }
}

impl From<MouseEvent> for Input {
    /// Convert [`crossterm::event::MouseEvent`] to [`Input`].
    fn from(mouse: MouseEvent) -> Self {
        let key = match mouse.kind {
            MouseEventKind::ScrollDown => Key::MouseScrollDown,
            MouseEventKind::ScrollUp => Key::MouseScrollUp,
            _ => Key::Null,
        };
        let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);
        let alt = mouse.modifiers.contains(KeyModifiers::ALT);
        Self { key, ctrl, alt }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crossterm::event::{KeyEventKind, KeyEventState};
    use crate::input::tests::input;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    fn mouse_event(kind: MouseEventKind, modifiers: KeyModifiers) -> MouseEvent {
        MouseEvent {
            kind,
            column: 1,
            row: 1,
            modifiers,
        }
    }

    #[test]
    fn key_to_input() {
        for (from, to) in [
            (
                key_event(KeyCode::Char('a'), KeyModifiers::empty()),
                input(Key::Char('a'), false, false),
            ),
            (
                key_event(KeyCode::Enter, KeyModifiers::empty()),
                input(Key::Enter, false, false),
            ),
            (
                key_event(KeyCode::Left, KeyModifiers::CONTROL),
                input(Key::Left, true, false),
            ),
            (
                key_event(KeyCode::Home, KeyModifiers::ALT),
                input(Key::Home, false, true),
            ),
            (
                key_event(KeyCode::F(1), KeyModifiers::ALT | KeyModifiers::CONTROL),
                input(Key::F(1), true, true),
            ),
            (
                key_event(KeyCode::NumLock, KeyModifiers::CONTROL),
                input(Key::Null, true, false),
            ),
        ] {
            assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn mouse_to_input() {
        for (from, to) in [
            (
                mouse_event(MouseEventKind::ScrollDown, KeyModifiers::empty()),
                input(Key::MouseScrollDown, false, false),
            ),
            (
                mouse_event(MouseEventKind::ScrollUp, KeyModifiers::CONTROL),
                input(Key::MouseScrollUp, true, false),
            ),
            (
                mouse_event(MouseEventKind::ScrollDown, KeyModifiers::ALT),
                input(Key::MouseScrollDown, false, true),
            ),
            (
                mouse_event(
                    MouseEventKind::ScrollUp,
                    KeyModifiers::CONTROL | KeyModifiers::ALT,
                ),
                input(Key::MouseScrollUp, true, true),
            ),
            (
                mouse_event(MouseEventKind::Moved, KeyModifiers::CONTROL),
                input(Key::Null, true, false),
            ),
        ] {
            assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn event_to_input() {
        for (from, to) in [
            (
                Event::Key(key_event(KeyCode::Char('a'), KeyModifiers::empty())),
                input(Key::Char('a'), false, false),
            ),
            (
                Event::Mouse(mouse_event(
                    MouseEventKind::ScrollDown,
                    KeyModifiers::empty(),
                )),
                input(Key::MouseScrollDown, false, false),
            ),
            (Event::FocusGained, input(Key::Null, false, false)),
        ] {
            assert_eq!(Input::from(from.clone()), to, "{:?} -> {:?}", from, to);
        }
    }

    // Regression for https://github.com/rhysd/tui-textarea/issues/14
    #[test]
    #[cfg(target_os = "windows")]
    fn press_ignore_on_windows() {
        let mut k = key_event(KeyCode::Char('a'), KeyModifiers::empty());
        k.kind = KeyEventKind::Release;
        let want = input(Key::Null, false, false);
        assert_eq!(Input::from(k.clone()), want, "{:?} -> {:?}", k, want);
    }
}
