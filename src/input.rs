#[cfg(any(feature = "crossterm", feature = "ratatui-crossterm"))]
use crate::crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
    MouseEvent as CrosstermMouseEvent, MouseEventKind as CrosstermMouseEventKind,
};
#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(any(feature = "crossterm", feature = "ratatui-crossterm"))]
use log::trace;
#[cfg(any(feature = "termion", feature = "ratatui-termion"))]
use termion::event::{Event as TermionEvent, Key as TermionKey, MouseEvent as TermionMouseEvent};

/// Backend-agnostic key input kind.
///
/// This type is marked as `#[non_exhaustive]` since more keys may be supported in the future.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
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
    /// Virtual key to scroll down by mouse
    MouseScrollDown,
    /// Virtual key to scroll up by mouse
    MouseScrollUp,
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
///
/// // `Input::from` can convert backend-native event into `Input`
/// let input = Input::from(event.clone());
/// // or `Into::into`
/// let input: Input = event.clone().into();
/// // Conversion from `KeyEvent` value is also available
/// if let Event::Key(key) = event {
///     let input = Input::from(key);
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
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct Input {
    /// Typed key.
    pub key: Key,
    /// Ctrl modifier key. `true` means Ctrl key was pressed.
    pub ctrl: bool,
    /// Alt modifier key. `true` means Alt key was pressed.
    pub alt: bool,
}

impl Default for Input {
    /// The default input is [`Key::Null`] without pressing any modifier keys, which means invalid input.
    fn default() -> Self {
        Input {
            key: Key::Null,
            ctrl: false,
            alt: false,
        }
    }
}

#[cfg(any(feature = "crossterm", feature = "ratatui-crossterm"))]
impl From<CrosstermEvent> for Input {
    /// Convert [`crossterm::event::Event`] to [`Input`].
    fn from(event: CrosstermEvent) -> Self {
        trace!("Cross Event:{:?}", event);
        match event {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Self::from(key),
            CrosstermEvent::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

#[cfg(any(feature = "crossterm", feature = "ratatui-crossterm"))]
impl From<KeyEvent> for Input {
    /// Convert [`crossterm::event::KeyEvent`] to [`Input`].
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

#[cfg(any(feature = "crossterm", feature = "ratatui-crossterm"))]
impl From<CrosstermMouseEvent> for Input {
    /// Convert [`crossterm::event::MouseEvent`] to [`Input`].
    fn from(mouse: CrosstermMouseEvent) -> Self {
        let key = match mouse.kind {
            CrosstermMouseEventKind::ScrollDown => Key::MouseScrollDown,
            CrosstermMouseEventKind::ScrollUp => Key::MouseScrollUp,
            _ => return Self::default(),
        };
        let ctrl = mouse.modifiers.contains(KeyModifiers::CONTROL);
        let alt = mouse.modifiers.contains(KeyModifiers::ALT);
        Self { key, ctrl, alt }
    }
}

#[cfg(any(feature = "termion", feature = "ratatui-termion"))]
impl From<TermionEvent> for Input {
    /// Convert [`termion::event::Event`] to [`Input`].
    fn from(event: TermionEvent) -> Self {
        match event {
            TermionEvent::Key(key) => Self::from(key),
            TermionEvent::Mouse(mouse) => Self::from(mouse),
            _ => Self::default(),
        }
    }
}

#[cfg(any(feature = "termion", feature = "ratatui-termion"))]
impl From<TermionKey> for Input {
    /// Convert [`termion::event::Key`] to [`Input`].
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

#[cfg(any(feature = "termion", feature = "ratatui-termion"))]
impl From<TermionMouseEvent> for Input {
    /// Convert [`termion::event::MouseEvent`] to [`Input`].
    fn from(mouse: TermionMouseEvent) -> Self {
        use termion::event::MouseButton;
        let key = match mouse {
            TermionMouseEvent::Press(MouseButton::WheelUp, ..) => Key::MouseScrollUp,
            TermionMouseEvent::Press(MouseButton::WheelDown, ..) => Key::MouseScrollDown,
            _ => return Self::default(),
        };
        Self {
            key,
            ctrl: false,
            alt: false,
        }
    }
}
