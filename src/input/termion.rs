use super::{Input, Key};
use termion::event::{Event, Key as KeyEvent, MouseEvent};

impl From<Event> for Input {
    /// Convert [`termion::event::Event`] to [`Input`].
    fn from(event: Event) -> Self {
        match event {
            Event::Key(key) => Self::from(key),
            Event::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

impl From<KeyEvent> for Input {
    /// Convert [`termion::event::Key`] to [`Input`].
    fn from(key: KeyEvent) -> Self {
        use KeyEvent::*;

        let mut ctrl = false;
        let mut alt = false;
        let key = match key {
            Char('\n' | '\r') => Key::Enter,
            Char(c) => Key::Char(c),
            Ctrl(c) => {
                ctrl = true;
                Key::Char(c)
            }
            Alt(c) => {
                alt = true;
                Key::Char(c)
            }
            Backspace => Key::Backspace,
            Left => Key::Left,
            Right => Key::Right,
            Up => Key::Up,
            Down => Key::Down,
            Home => Key::Home,
            End => Key::End,
            PageUp => Key::PageUp,
            PageDown => Key::PageDown,
            BackTab => Key::Tab,
            Delete => Key::Delete,
            Esc => Key::Esc,
            F(x) => Key::F(x),
            _ => Key::Null,
        };

        Input { key, ctrl, alt }
    }
}

impl From<MouseEvent> for Input {
    /// Convert [`termion::event::MouseEvent`] to [`Input`].
    fn from(mouse: MouseEvent) -> Self {
        use termion::event::MouseButton;
        let key = match mouse {
            MouseEvent::Press(MouseButton::WheelUp, ..) => Key::MouseScrollUp,
            MouseEvent::Press(MouseButton::WheelDown, ..) => Key::MouseScrollDown,
            _ => return Self::default(),
        };
        Self {
            key,
            ctrl: false,
            alt: false,
        }
    }
}
