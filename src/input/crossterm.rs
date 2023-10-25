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
            _ => return Self::default(),
        };
        let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);
        let alt = mouse.modifiers.contains(KeyModifiers::ALT);
        Self { key, ctrl, alt }
    }
}
