use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Tab,
    Delete,
    Home,
    End,
    Null,
}

#[derive(Debug)]
pub struct Input {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            key: Key::Null,
            ctrl: false,
            alt: false,
        }
    }
}

impl From<Event> for Input {
    fn from(event: Event) -> Self {
        if let Event::Key(key) = event {
            Self::from(key)
        } else {
            Self::default()
        }
    }
}

impl From<KeyEvent> for Input {
    fn from(key: KeyEvent) -> Self {
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
            _ => Key::Null,
        };
        Self { key, ctrl, alt }
    }
}
