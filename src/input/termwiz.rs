use super::{Input, Key};
use termwiz::input::{InputEvent, KeyEvent, MouseButtons, MouseEvent, PixelMouseEvent};

impl From<InputEvent> for Input {
    /// Convert [`termwiz::input::InputEvent`] to [`Input`].
    fn from(input: InputEvent) -> Self {
        match input {
            InputEvent::Key(key) => Self::from(key),
            InputEvent::Mouse(mouse) => Self::from(mouse),
            InputEvent::PixelMouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

impl From<KeyEvent> for Input {
    /// Convert [`termwiz::input::KeyEvent`] to [`Input`].
    fn from(key: KeyEvent) -> Self {
        use termwiz::input::{KeyCode, Modifiers};

        let KeyEvent { key, modifiers } = key;
        let key = match key {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Tab => Key::Tab,
            KeyCode::Enter => Key::Enter,
            KeyCode::Escape => Key::Esc,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::End => Key::End,
            KeyCode::Home => Key::Home,
            KeyCode::LeftArrow => Key::Left,
            KeyCode::RightArrow => Key::Right,
            KeyCode::UpArrow => Key::Up,
            KeyCode::DownArrow => Key::Down,
            KeyCode::Delete => Key::Delete,
            KeyCode::Function(x) => Key::F(x),
            _ => Key::Null,
        };
        let ctrl = modifiers.contains(Modifiers::CTRL);
        let alt = modifiers.contains(Modifiers::ALT);

        Self { key, ctrl, alt }
    }
}

impl From<MouseButtons> for Key {
    /// Convert [`termwiz::input::MouseButtons`] to [`Key`].
    fn from(buttons: MouseButtons) -> Self {
        if buttons.contains(MouseButtons::VERT_WHEEL) {
            if buttons.contains(MouseButtons::WHEEL_POSITIVE) {
                Key::MouseScrollUp
            } else {
                Key::MouseScrollDown
            }
        } else {
            Key::Null
        }
    }
}

impl From<MouseEvent> for Input {
    /// Convert [`termwiz::input::MouseEvent`] to [`Input`].
    fn from(mouse: MouseEvent) -> Self {
        use termwiz::input::Modifiers;

        let MouseEvent {
            mouse_buttons,
            modifiers,
            ..
        } = mouse;
        let key = Key::from(mouse_buttons);
        let ctrl = modifiers.contains(Modifiers::CTRL);
        let alt = modifiers.contains(Modifiers::ALT);

        Self { key, ctrl, alt }
    }
}

impl From<PixelMouseEvent> for Input {
    /// Convert [`termwiz::input::PixelMouseEvent`] to [`Input`].
    fn from(mouse: PixelMouseEvent) -> Self {
        use termwiz::input::Modifiers;

        let PixelMouseEvent {
            mouse_buttons,
            modifiers,
            ..
        } = mouse;
        let key = Key::from(mouse_buttons);
        let ctrl = modifiers.contains(Modifiers::CTRL);
        let alt = modifiers.contains(Modifiers::ALT);

        Self { key, ctrl, alt }
    }
}
