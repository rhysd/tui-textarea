use super::{Input, Key};
use termion::event::{Event, Key as KeyEvent, MouseButton, MouseEvent};

impl From<Event> for Input {
    /// Convert [`termion::event::Event`] into [`Input`].
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => Self::from(key),
            Event::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

impl From<KeyEvent> for Input {
    /// Convert [`termion::event::Key`] into [`Input`].
    fn from(key: KeyEvent) -> Self {
        let mut ctrl = false;
        let mut alt = false;
        let shift = false;
        let key = match key {
            KeyEvent::Char('\n' | '\r') => Key::Enter,
            KeyEvent::Char(c) => Key::Char(c),
            KeyEvent::Ctrl(c) => {
                ctrl = true;
                Key::Char(c)
            }
            KeyEvent::Alt(c) => {
                alt = true;
                Key::Char(c)
            }
            KeyEvent::Backspace => Key::Backspace,
            KeyEvent::Left => Key::Left,
            KeyEvent::Right => Key::Right,
            KeyEvent::Up => Key::Up,
            KeyEvent::Down => Key::Down,
            KeyEvent::Home => Key::Home,
            KeyEvent::End => Key::End,
            KeyEvent::PageUp => Key::PageUp,
            KeyEvent::PageDown => Key::PageDown,
            KeyEvent::BackTab => Key::Tab,
            KeyEvent::Delete => Key::Delete,
            KeyEvent::Esc => Key::Esc,
            KeyEvent::F(x) => Key::F(x),
            _ => Key::Null,
        };

        Input {
            key,
            ctrl,
            alt,
            shift,
        }
    }
}

impl From<MouseButton> for Key {
    /// Convert [`termion::event::MouseButton`] into [`Key`].
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::WheelUp => Key::MouseScrollUp,
            MouseButton::WheelDown => Key::MouseScrollDown,
            _ => Key::Null,
        }
    }
}

impl From<MouseEvent> for Input {
    /// Convert [`termion::event::MouseEvent`] into [`Input`].
    fn from(mouse: MouseEvent) -> Self {
        let key = if let MouseEvent::Press(button, ..) = mouse {
            Key::from(button)
        } else {
            Key::Null
        };
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::tests::input;

    #[test]
    fn key_to_input() {
        for (from, to) in [
            (KeyEvent::Char('a'), input(Key::Char('a'), false, false)),
            (KeyEvent::Ctrl('a'), input(Key::Char('a'), true, false)),
            (KeyEvent::Alt('a'), input(Key::Char('a'), false, true)),
            (KeyEvent::Char('\n'), input(Key::Enter, false, false)),
            (KeyEvent::Char('\r'), input(Key::Enter, false, false)),
            (KeyEvent::F(1), input(Key::F(1), false, false)),
            (KeyEvent::BackTab, input(Key::Tab, false, false)),
            (KeyEvent::Null, input(Key::Null, false, false)),
        ] {
            assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn mouse_to_input() {
        for (from, to) in [
            (
                MouseEvent::Press(MouseButton::WheelDown, 1, 1),
                input(Key::MouseScrollDown, false, false),
            ),
            (
                MouseEvent::Press(MouseButton::WheelUp, 1, 1),
                input(Key::MouseScrollUp, false, false),
            ),
            (
                MouseEvent::Press(MouseButton::Left, 1, 1),
                input(Key::Null, false, false),
            ),
            (MouseEvent::Release(1, 1), input(Key::Null, false, false)),
            (MouseEvent::Hold(1, 1), input(Key::Null, false, false)),
        ] {
            assert_eq!(Input::from(from), to, "{:?} -> {:?}", from, to);
        }
    }

    #[test]
    fn event_to_input() {
        for (from, to) in [
            (
                Event::Key(KeyEvent::Char('a')),
                input(Key::Char('a'), false, false),
            ),
            (
                Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, 1, 1)),
                input(Key::MouseScrollDown, false, false),
            ),
            (Event::Unsupported(vec![]), input(Key::Null, false, false)),
        ] {
            assert_eq!(Input::from(from.clone()), to, "{:?} -> {:?}", from, to);
        }
    }
}
