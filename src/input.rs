#[cfg(feature = "crossterm")]
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
#[cfg(feature = "termion")]
use termion::event::{Event as TerimonEvent, Key as TermionKey};

/// Backend-agnostic key input kind.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Key {
    /// Normal letter key input.
    Char(char),
    /// F1, F2, F3, ... keys.
    F(u8),
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
    Esc,
    /// An invalid key input (this key is always ignored by [`TextArea`](crate::TextArea)).
    Null,
}

/// Backend-agnostic key input type.
///
/// When `crossterm` and/or `termion` features are enabled, converting their key input types into this `Input` type is defined.
/// ```no_run
/// use tui_textarea::{TextArea, Input, Key};
/// use crossterm::event::{Event, read};
///
/// let event = read().unwrap();
/// let input = Input::from(event);
/// if let Event::Key(key) = event {
///     let input = Input::from(key); // Conversion from `KeyEvent` value is also available
/// }
/// ```
///
/// Creating `Input` instance directly can cause backend-agnostic input as follows.
///
/// ```
/// use tui_textarea::{TextArea, Input, Key};
///
/// let mut textarea = TextArea::default();
///
/// // Input Ctrl+A
/// textarea.input(Input {
///     key: Key::Char('a'),
///     ctrl: true,
///     alt: false,
/// });
/// ```
#[derive(Debug, Clone)]
pub struct Input {
    /// Typed key.
    pub key: Key,
    /// Ctrl modifier key. `true` means Ctrl key was pressed.
    pub ctrl: bool,
    /// Alt modifier key. `true` means Alt key was pressed.
    pub alt: bool,
}

impl Default for Input {
    /// The default input is [`Key::Null`] without pressing Ctrl nor Alt, which means invalid input.
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
            KeyCode::Esc => Key::Esc,
            KeyCode::F(x) => Key::F(x),
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
            Esc => Key::Esc,
            F(x) => Key::F(x),
            _ => Key::Null,
        };

        Input { key, ctrl, alt }
    }
}
