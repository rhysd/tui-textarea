#[cfg(feature = "crossterm")]
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
#[cfg(feature = "termion")]
use termion::event::{Event as TerimonEvent, Key as TermionKey};

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
    PageUp,
    PageDown,
    Null,
}

#[derive(Debug, Clone)]
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

#[cfg(feature = "crossterm")]
impl From<CrosstermEvent> for Input {
    fn from(event: CrosstermEvent) -> Self {
        if let CrosstermEvent::Key(key) = event {
            Self::from(key)
        } else {
            Self::default()
        }
    }
}

#[cfg(feature = "crossterm")]
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
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            _ => Key::Null,
        };
        Self { key, ctrl, alt }
    }
}

#[cfg(feature = "termion")]
impl From<TerimonEvent> for Input {
    fn from(event: TerimonEvent) -> Self {
        if let TerimonEvent::Key(key) = event {
            Self::from(key)
        } else {
            Self::default()
        }
    }
}

#[cfg(feature = "termion")]
impl From<TermionKey> for Input {
    fn from(key: TermionKey) -> Self {
        use TermionKey::*;

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
            _ => Key::Null,
        };

        Input { key, ctrl, alt }
    }
}
