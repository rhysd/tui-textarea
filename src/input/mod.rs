#[cfg(any(feature = "crossterm", feature = "tuirs-crossterm"))]
mod crossterm;
#[cfg(any(feature = "termion", feature = "tuirs-termion"))]
mod termion;
#[cfg(feature = "termwiz")]
mod termwiz;

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;

/// Backend-agnostic key input kind.
///
/// This type is marked as `#[non_exhaustive]` since more keys may be supported in the future.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum Key {
    /// Normal letter key input
    Char(char),
    /// F1, F2, F3, ... keys
    F(u8),
    /// Backspace key
    Backspace,
    /// Enter or return key
    Enter,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Tab key
    Tab,
    /// Delete key
    Delete,
    /// Home key
    Home,
    /// End key
    End,
    /// Page up key
    PageUp,
    /// Page down key
    PageDown,
    /// Escape key
    Esc,
    /// Virtual key to scroll down by mouse
    MouseScrollDown,
    /// Virtual key to scroll up by mouse
    MouseScrollUp,
    /// An invalid key input (this key is always ignored by [`TextArea`](crate::TextArea))
    Null,
}

impl Default for Key {
    fn default() -> Self {
        Key::Null
    }
}

/// Backend-agnostic key input type.
///
/// When `crossterm`, `termion`, `termwiz` features are enabled, converting respective key input types into this
/// `Input` type is defined.
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
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct Input {
    /// Typed key.
    pub key: Key,
    /// Ctrl modifier key. `true` means Ctrl key was pressed.
    pub ctrl: bool,
    /// Alt modifier key. `true` means Alt key was pressed.
    pub alt: bool,
}
