use crate::cursor::CursorMove;
use crate::highlight::LineHighlighter;
use crate::history::{Edit, EditKind, History};
use crate::input::{Input, Key};
use crate::scroll::Scrolling;
#[cfg(feature = "search")]
use crate::search::Search;
use crate::tui::layout::Alignment;
use crate::tui::style::{Modifier, Style};
use crate::tui::text::Spans;
use crate::tui::widgets::{Block, Widget};
use crate::util::spaces;
use crate::widget::{Renderer, Viewport};
use crate::word::{find_word_end_forward, find_word_start_backward};

/// A type to manage state of textarea.
///
/// [`TextArea::default`] creates an empty textarea. [`TextArea::new`] creates a textarea with given text lines.
/// [`TextArea::from`] creates a textarea from an iterator of lines. [`TextArea::input`] handles key input.
/// [`TextArea::widget`] builds a widget to render. And [`TextArea::lines`] returns line texts.
/// ```
/// use tui_textarea::{TextArea, Input, Key};
///
/// let mut textarea = TextArea::default();
///
/// // Input 'a'
/// let input = Input { key: Key::Char('a'), ctrl: false, alt: false };
/// textarea.input(input);
///
/// // Get widget to render.
/// let widget = textarea.widget();
///
/// // Get lines as String.
/// println!("Lines: {:?}", textarea.lines());
/// ```
#[derive(Clone)]
pub struct TextArea<'a> {
    lines: Vec<String>,
    block: Option<Block<'a>>,
    style: Style,
    cursor: (usize, usize), // 0-base
    tab_len: u8,
    hard_tab_indent: bool,
    history: History,
    cursor_line_style: Style,
    line_number_style: Option<Style>,
    pub(crate) viewport: Viewport,
    cursor_style: Style,
    yank: String,
    #[cfg(feature = "search")]
    search: Search,
    alignment: Alignment,
    pub(crate) placeholder: Option<String>,
    pub(crate) placeholder_style: Option<Style>,
}

/// Convert any iterator whose elements can be converted into [`String`] into [`TextArea`]. Each [`String`] element is
/// handled as line. Ensure that the strings don't contain any newlines. This method is useful to create [`TextArea`]
/// from [`std::str::Lines`].
/// ```
/// use tui_textarea::TextArea;
///
/// // From `String`
/// let text = "hello\nworld";
/// let textarea = TextArea::from(text.lines());
/// assert_eq!(textarea.lines(), ["hello", "world"]);
///
/// // From array of `&str`
/// let textarea = TextArea::from(["hello", "world"]);
/// assert_eq!(textarea.lines(), ["hello", "world"]);
///
/// // From slice of `&str`
/// let slice = &["hello", "world"];
/// let textarea = TextArea::from(slice.iter().copied());
/// assert_eq!(textarea.lines(), ["hello", "world"]);
/// ```
impl<'a, I> From<I> for TextArea<'a>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    fn from(i: I) -> Self {
        Self::new(i.into_iter().map(|s| s.into()).collect::<Vec<String>>())
    }
}

/// Collect line texts from iterator as [`TextArea`]. It is useful when creating a textarea with text read from a file.
/// [`Iterator::collect`] handles errors which may happen on reading each lines. The following example reads text from
/// a file efficiently line-by-line.
/// ```no_run
/// use std::fs;
/// use std::io::{self, BufRead};
/// use std::path::Path;
/// use tui_textarea::TextArea;
///
/// fn read_from_file<'a>(path: impl AsRef<Path>) -> io::Result<TextArea<'a>> {
///     let file = fs::File::open(path.as_ref())?;
///     io::BufReader::new(file).lines().collect()
/// }
/// ```
impl<'a, S: Into<String>> FromIterator<S> for TextArea<'a> {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        iter.into()
    }
}

/// Create [`TextArea`] instance with empty text content.
/// ```
/// use tui_textarea::TextArea;
///
/// let textarea = TextArea::default();
/// assert_eq!(textarea.lines(), [""]);
/// assert!(textarea.is_empty());
/// ```
impl<'a> Default for TextArea<'a> {
    fn default() -> Self {
        Self::new(vec![String::new()])
    }
}

impl<'a> TextArea<'a> {
    /// Create [`TextArea`] instance with given lines. If you have value other than `Vec<String>`, [`TextArea::from`]
    /// may be more useful.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let lines = vec!["hello".to_string(), "...".to_string(), "goodbye".to_string()];
    /// let textarea = TextArea::new(lines);
    /// assert_eq!(textarea.lines(), ["hello", "...", "goodbye"]);
    /// ```
    pub fn new(mut lines: Vec<String>) -> Self {
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            block: None,
            style: Style::default(),
            cursor: (0, 0),
            tab_len: 4,
            hard_tab_indent: false,
            history: History::new(50),
            cursor_line_style: Style::default().add_modifier(Modifier::UNDERLINED),
            line_number_style: None,
            viewport: Viewport::default(),
            cursor_style: Style::default().add_modifier(Modifier::REVERSED),
            yank: String::new(),
            #[cfg(feature = "search")]
            search: Search::default(),
            alignment: Alignment::Left,
            placeholder: None,
            placeholder_style: None,
        }
    }

    /// Handle a key input with default key mappings. For default key mappings, see the table in
    /// [the module document](./index.html).
    /// `crossterm` and `termion` features enable conversion from their own key event types into [`Input`] so this
    /// method can take the event values directly.
    /// This method returns if the input modified text contents or not in the textarea.
    /// ```ignore
    /// use tui_textarea::{TextArea, Key, Input};
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// // Handle crossterm key events
    /// let event: crossterm::event::Event = ...;
    /// textarea.input(event);
    /// if let crossterm::event::Event::Key(key) = event {
    ///     textarea.input(key);
    /// }
    ///
    /// // Handle termion key events
    /// let event: termion::event::Event = ...;
    /// textarea.input(event);
    /// if let termion::event::Event::Key(key) = event {
    ///     textarea.input(key);
    /// }
    ///
    /// // Handle backend-agnostic key input
    /// let input = Input { key: Key::Char('a'), ctrl: false, alt: false };
    /// let modified = textarea.input(input);
    /// assert!(modified);
    /// ```
    pub fn input(&mut self, input: impl Into<Input>) -> bool {
        let input = input.into();
        let modified = match input {
            Input {
                key: Key::Char('m'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Char('\n' | '\r'),
                ctrl: false,
                alt: false,
            }
            | Input {
                key: Key::Enter, ..
            } => {
                self.insert_newline();
                true
            }
            Input {
                key: Key::Char(c),
                ctrl: false,
                alt: false,
            } => {
                self.insert_char(c);
                true
            }
            Input {
                key: Key::Tab,
                ctrl: false,
                alt: false,
            } => self.insert_tab(),
            Input {
                key: Key::Char('h'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Backspace,
                ctrl: false,
                alt: false,
            } => self.delete_char(),
            Input {
                key: Key::Char('d'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Delete,
                ctrl: false,
                alt: false,
            } => self.delete_next_char(),
            Input {
                key: Key::Char('k'),
                ctrl: true,
                alt: false,
            } => self.delete_line_by_end(),
            Input {
                key: Key::Char('j'),
                ctrl: true,
                alt: false,
            } => self.delete_line_by_head(),
            Input {
                key: Key::Char('w'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Char('h'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Backspace,
                ctrl: false,
                alt: true,
            } => self.delete_word(),
            Input {
                key: Key::Delete,
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Char('d'),
                ctrl: false,
                alt: true,
            } => self.delete_next_word(),
            Input {
                key: Key::Char('n'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Down,
                ctrl: false,
                alt: false,
            } => {
                self.move_cursor(CursorMove::Down);
                false
            }
            Input {
                key: Key::Char('p'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Up,
                ctrl: false,
                alt: false,
            } => {
                self.move_cursor(CursorMove::Up);
                false
            }
            Input {
                key: Key::Char('f'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Right,
                ctrl: false,
                alt: false,
            } => {
                self.move_cursor(CursorMove::Forward);
                false
            }
            Input {
                key: Key::Char('b'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::Left,
                ctrl: false,
                alt: false,
            } => {
                self.move_cursor(CursorMove::Back);
                false
            }
            Input {
                key: Key::Char('a'),
                ctrl: true,
                alt: false,
            }
            | Input { key: Key::Home, .. }
            | Input {
                key: Key::Left | Key::Char('b'),
                ctrl: true,
                alt: true,
            } => {
                self.move_cursor(CursorMove::Head);
                false
            }
            Input {
                key: Key::Char('e'),
                ctrl: true,
                alt: false,
            }
            | Input { key: Key::End, .. }
            | Input {
                key: Key::Right | Key::Char('f'),
                ctrl: true,
                alt: true,
            } => {
                self.move_cursor(CursorMove::End);
                false
            }
            Input {
                key: Key::Char('<'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Up | Key::Char('p'),
                ctrl: true,
                alt: true,
            } => {
                self.move_cursor(CursorMove::Top);
                false
            }
            Input {
                key: Key::Char('>'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Down | Key::Char('n'),
                ctrl: true,
                alt: true,
            } => {
                self.move_cursor(CursorMove::Bottom);
                false
            }
            Input {
                key: Key::Char('f'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Right,
                ctrl: true,
                alt: false,
            } => {
                self.move_cursor(CursorMove::WordForward);
                false
            }
            Input {
                key: Key::Char('b'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Left,
                ctrl: true,
                alt: false,
            } => {
                self.move_cursor(CursorMove::WordBack);
                false
            }
            Input {
                key: Key::Char(']'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Char('n'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Down,
                ctrl: true,
                alt: false,
            } => {
                self.move_cursor(CursorMove::ParagraphForward);
                false
            }
            Input {
                key: Key::Char('['),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Char('p'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::Up,
                ctrl: true,
                alt: false,
            } => {
                self.move_cursor(CursorMove::ParagraphBack);
                false
            }
            Input {
                key: Key::Char('u'),
                ctrl: true,
                alt: false,
            } => self.undo(),
            Input {
                key: Key::Char('r'),
                ctrl: true,
                alt: false,
            } => self.redo(),
            Input {
                key: Key::Char('y'),
                ctrl: true,
                alt: false,
            } => self.paste(),
            Input {
                key: Key::Char('v'),
                ctrl: true,
                alt: false,
            }
            | Input {
                key: Key::PageDown, ..
            } => {
                self.scroll(Scrolling::PageDown);
                false
            }
            Input {
                key: Key::Char('v'),
                ctrl: false,
                alt: true,
            }
            | Input {
                key: Key::PageUp, ..
            } => {
                self.scroll(Scrolling::PageUp);
                false
            }
            Input {
                key: Key::MouseScrollDown,
                ..
            } => {
                self.scroll((1, 0));
                false
            }
            Input {
                key: Key::MouseScrollUp,
                ..
            } => {
                self.scroll((-1, 0));
                false
            }
            _ => false,
        };

        // Check invariants
        debug_assert!(!self.lines.is_empty(), "no line after {:?}", input);
        let (r, c) = self.cursor;
        debug_assert!(
            self.lines.len() > r,
            "cursor {:?} exceeds max lines {} after {:?}",
            self.cursor,
            self.lines.len(),
            input,
        );
        debug_assert!(
            self.lines[r].chars().count() >= c,
            "cursor {:?} exceeds max col {} at line {:?} after {:?}",
            self.cursor,
            self.lines[r].chars().count(),
            self.lines[r],
            input,
        );

        modified
    }

    /// Handle a key input without default key mappings. This method handles only
    ///
    /// - Single character input without modifier keys
    /// - Tab
    /// - Enter
    /// - Backspace
    /// - Delete
    ///
    /// This method returns if the input modified text contents or not in the textarea.
    ///
    /// This method is useful when you want to define your own key mappings and don't want default key mappings.
    /// See 'Define your own key mappings' section in [the module document](./index.html).
    pub fn input_without_shortcuts(&mut self, input: impl Into<Input>) -> bool {
        match input.into() {
            Input {
                key: Key::Char(c),
                ctrl: false,
                alt: false,
            } => {
                self.insert_char(c);
                true
            }
            Input {
                key: Key::Tab,
                ctrl: false,
                alt: false,
            } => self.insert_tab(),
            Input {
                key: Key::Backspace,
                ..
            } => self.delete_char(),
            Input {
                key: Key::Delete, ..
            } => self.delete_next_char(),
            Input {
                key: Key::Enter, ..
            } => {
                self.insert_newline();
                true
            }
            Input {
                key: Key::MouseScrollDown,
                ..
            } => {
                self.scroll((1, 0));
                false
            }
            Input {
                key: Key::MouseScrollUp,
                ..
            } => {
                self.scroll((-1, 0));
                false
            }
            _ => false,
        }
    }

    fn push_history(&mut self, kind: EditKind, cursor_before: (usize, usize)) {
        let edit = Edit::new(kind, cursor_before, self.cursor);
        self.history.push(edit);
    }

    /// Insert a single character at current cursor position.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.insert_char('a');
    /// assert_eq!(textarea.lines(), ["a"]);
    /// ```
    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(i, c);
        self.cursor.1 += 1;
        self.push_history(EditKind::InsertChar(c, i), (row, col));
    }

    /// Insert a string at current cursor position. Currently the string must not contain any newlines. This method
    /// returns if some text was inserted or not in the textarea.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.insert_str("hello");
    /// assert_eq!(textarea.lines(), ["hello"]);
    /// ```
    pub fn insert_str<S: Into<String>>(&mut self, s: S) -> bool {
        let s = s.into();
        if s.is_empty() {
            return false;
        }

        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        debug_assert!(
            !line.contains('\n'),
            "string given to insert_str must not contain newline: {:?}",
            line,
        );

        let i = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert_str(i, &s);

        self.cursor.1 += s.chars().count();
        self.push_history(EditKind::Insert(s, i), (row, col));
        true
    }

    /// Delete a string in current cursor line. The `chars` parameter means number of characters, not a byte length of
    /// the string. This method returns if some text was deleted or not in the textarea.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["🐱🐶🐰🐮"]);
    ///
    /// textarea.delete_str(1, 2);
    /// assert_eq!(textarea.lines(), ["🐱🐮"]);
    /// ```
    pub fn delete_str(&mut self, col: usize, chars: usize) -> bool {
        if chars == 0 {
            return false;
        }

        let cursor_before = self.cursor;
        let row = cursor_before.0;
        let line = &mut self.lines[row];
        if let Some((i, _)) = line.char_indices().nth(col) {
            let bytes = line[i..]
                .char_indices()
                .nth(chars)
                .map(|(i, _)| i)
                .unwrap_or_else(|| line[i..].len());
            let removed = line[i..i + bytes].to_string();
            line.replace_range(i..i + bytes, "");

            self.cursor = (row, col);
            self.push_history(EditKind::Remove(removed.clone(), i), cursor_before);
            self.yank = removed;
            true
        } else {
            false
        }
    }

    /// Insert a tab at current cursor position. Note that this method does nothing when the tab length is 0. This
    /// method returns if a tab string was inserted or not in the textarea.
    /// textarea.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["hi"]);
    ///
    /// textarea.insert_tab();
    /// assert_eq!(textarea.lines(), ["    hi"]);
    /// ```
    pub fn insert_tab(&mut self) -> bool {
        if self.tab_len == 0 {
            return false;
        }
        let tab = if self.hard_tab_indent {
            "\t"
        } else {
            let len = self.tab_len - (self.cursor.1 % self.tab_len as usize) as u8;
            spaces(len)
        };
        self.insert_str(tab)
    }

    /// Insert a newline at current cursor position.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["hi"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.insert_newline();
    /// assert_eq!(textarea.lines(), ["h", "i"]);
    /// ```
    pub fn insert_newline(&mut self) {
        let (row, col) = self.cursor;
        let line = &mut self.lines[row];
        let idx = line
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let next_line = line[idx..].to_string();
        line.truncate(idx);

        self.lines.insert(row + 1, next_line);
        self.cursor = (row + 1, 0);
        self.push_history(EditKind::InsertNewline(idx), (row, col));
    }

    /// Delete a newline from **head** of current cursor line. This method returns if a newline was deleted or not in
    /// the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["hello", "world"]);
    ///
    /// textarea.move_cursor(CursorMove::Down);
    /// textarea.delete_newline();
    /// assert_eq!(textarea.lines(), ["helloworld"]);
    /// ```
    pub fn delete_newline(&mut self) -> bool {
        let (row, col) = self.cursor;
        if row == 0 {
            return false;
        }

        let line = self.lines.remove(row);
        let prev_line = &mut self.lines[row - 1];
        let prev_line_end = prev_line.len();

        self.cursor = (row - 1, prev_line.chars().count());
        prev_line.push_str(&line);
        self.push_history(EditKind::DeleteNewline(prev_line_end), (row, col));
        true
    }

    /// Delete one character before cursor. When the cursor is at head of line, the newline before the cursor will be
    /// removed. This method returns if some text was deleted or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.delete_char();
    /// assert_eq!(textarea.lines(), ["bc"]);
    /// ```
    pub fn delete_char(&mut self) -> bool {
        let (row, col) = self.cursor;
        if col == 0 {
            return self.delete_newline();
        }

        let line = &mut self.lines[row];
        if let Some((i, c)) = line.char_indices().nth(col - 1) {
            line.remove(i);
            self.cursor.1 -= 1;
            self.push_history(EditKind::DeleteChar(c, i), (row, col));
            true
        } else {
            false
        }
    }

    /// Delete one character next to cursor. When the cursor is at end of line, the newline next to the cursor will be
    /// removed. This method returns if a character was deleted or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.delete_next_char();
    /// assert_eq!(textarea.lines(), ["ac"]);
    /// ```
    pub fn delete_next_char(&mut self) -> bool {
        let before = self.cursor;
        self.move_cursor(CursorMove::Forward);
        if before == self.cursor {
            return false; // Cursor didn't move, meant no character at next of cursor.
        }
        self.delete_char()
    }

    /// Delete string from cursor to end of the line. When the cursor is at end of line, the newline next to the cursor
    /// is removed. This method returns if some text was deleted or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abcde"]);
    ///
    /// // Move to 'c'
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Forward);
    ///
    /// textarea.delete_line_by_end();
    /// assert_eq!(textarea.lines(), ["ab"]);
    /// ```
    pub fn delete_line_by_end(&mut self) -> bool {
        if self.delete_str(self.cursor.1, usize::MAX) {
            return true;
        }
        self.delete_next_char() // At the end of the line. Try to delete next line
    }

    /// Delete string from cursor to head of the line. When the cursor is at head of line, the newline before the cursor
    /// will be removed. This method returns if some text was deleted or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abcde"]);
    ///
    /// // Move to 'c'
    /// textarea.move_cursor(CursorMove::Forward);
    /// textarea.move_cursor(CursorMove::Forward);
    ///
    /// textarea.delete_line_by_head();
    /// assert_eq!(textarea.lines(), ["cde"]);
    /// ```
    pub fn delete_line_by_head(&mut self) -> bool {
        if self.delete_str(0, self.cursor.1) {
            return true;
        }
        self.delete_newline()
    }

    /// Delete a word before cursor. Word boundary appears at spaces, punctuations, and others. For example `fn foo(a)`
    /// consists of words `fn`, `foo`, `(`, `a`, `)`. When the cursor is at head of line, the newline before the cursor
    /// will be removed.
    ///
    /// This method returns if some text was deleted or not in the textarea.
    ///
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["aaa bbb ccc"]);
    ///
    /// textarea.move_cursor(CursorMove::End);
    ///
    /// textarea.delete_word();
    /// assert_eq!(textarea.lines(), ["aaa bbb "]);
    /// textarea.delete_word();
    /// assert_eq!(textarea.lines(), ["aaa "]);
    /// ```
    pub fn delete_word(&mut self) -> bool {
        let (r, c) = self.cursor;
        if let Some(col) = find_word_start_backward(&self.lines[r], c) {
            self.delete_str(col, c - col)
        } else if c > 0 {
            self.delete_str(0, c)
        } else {
            self.delete_newline()
        }
    }

    /// Delete a word next to cursor. Word boundary appears at spaces, punctuations, and others. For example `fn foo(a)`
    /// consists of words `fn`, `foo`, `(`, `a`, `)`. When the cursor is at end of line, the newline next to the cursor
    /// will be removed.
    ///
    /// This method returns if some text was deleted or not in the textarea.
    ///
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["aaa bbb ccc"]);
    ///
    /// textarea.delete_next_word();
    /// assert_eq!(textarea.lines(), [" bbb ccc"]);
    /// textarea.delete_next_word();
    /// assert_eq!(textarea.lines(), [" ccc"]);
    /// ```
    pub fn delete_next_word(&mut self) -> bool {
        let (r, c) = self.cursor;
        let line = &self.lines[r];
        if let Some(col) = find_word_end_forward(line, c) {
            self.delete_str(c, col - c)
        } else {
            let end_col = line.chars().count();
            if c < end_col {
                self.delete_str(c, end_col - c)
            } else if r + 1 < self.lines.len() {
                self.cursor = (r + 1, 0);
                self.delete_newline()
            } else {
                false
            }
        }
    }

    /// Paste a string previously deleted by [`TextArea::delete_line_by_head`], [`TextArea::delete_line_by_end`],
    /// [`TextArea::delete_word`], [`TextArea::delete_next_word`]. This method returns if some text was inserted or not
    /// in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["aaa bbb ccc"]);
    ///
    /// textarea.delete_next_word();
    /// textarea.move_cursor(CursorMove::End);
    /// textarea.paste();
    /// assert_eq!(textarea.lines(), [" bbb cccaaa"]);
    /// ```
    pub fn paste(&mut self) -> bool {
        self.insert_str(self.yank.to_string())
    }

    /// Move the cursor to the position specified by the [`CursorMove`] parameter. For each kind of cursor moves, see
    /// the document of [`CursorMove`].
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc", "def"]);
    ///
    /// textarea.move_cursor(CursorMove::Forward);
    /// assert_eq!(textarea.cursor(), (0, 1));
    /// textarea.move_cursor(CursorMove::Down);
    /// assert_eq!(textarea.cursor(), (1, 1));
    /// ```
    pub fn move_cursor(&mut self, m: CursorMove) {
        if let Some(cursor) = m.next_cursor(self.cursor, &self.lines, &self.viewport) {
            self.cursor = cursor;
        }
    }

    /// Undo the last modification. This method returns if the undo modified text contents or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc def"]);
    ///
    /// textarea.delete_next_word();
    /// assert_eq!(textarea.lines(), [" def"]);
    /// textarea.undo();
    /// assert_eq!(textarea.lines(), ["abc def"]);
    /// ```
    pub fn undo(&mut self) -> bool {
        if let Some(cursor) = self.history.undo(&mut self.lines) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// Redo the last undo change. This method returns if the redo modified text contents or not in the textarea.
    /// ```
    /// use tui_textarea::{TextArea, CursorMove};
    ///
    /// let mut textarea = TextArea::from(["abc def"]);
    ///
    /// textarea.delete_next_word();
    /// assert_eq!(textarea.lines(), [" def"]);
    /// textarea.undo();
    /// assert_eq!(textarea.lines(), ["abc def"]);
    /// textarea.redo();
    /// assert_eq!(textarea.lines(), [" def"]);
    /// ```
    pub fn redo(&mut self) -> bool {
        if let Some(cursor) = self.history.redo(&mut self.lines) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    pub(crate) fn line_spans<'b>(&'b self, line: &'b str, row: usize, lnum_len: u8) -> Spans<'b> {
        let mut hl = LineHighlighter::new(line, self.cursor_style, self.tab_len);

        if let Some(style) = self.line_number_style {
            hl.line_number(row, lnum_len, style);
        }

        if row == self.cursor.0 {
            hl.cursor_line(self.cursor.1, self.cursor_line_style);
        }

        #[cfg(feature = "search")]
        if let Some(matches) = self.search.matches(line) {
            hl.search(matches, self.search.style);
        }

        hl.into_spans()
    }

    /// Build a tui-rs widget to render the current state of the textarea. The widget instance returned from this
    /// method can be rendered with [`tui::terminal::Frame::render_widget`].
    /// ```no_run
    /// use tui::backend::CrosstermBackend;
    /// use tui::layout::{Constraint, Direction, Layout};
    /// use tui::Terminal;
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// let layout = Layout::default()
    ///     .direction(Direction::Vertical)
    ///     .constraints([Constraint::Min(1)].as_ref());
    /// let backend = CrosstermBackend::new(std::io::stdout());
    /// let mut term = Terminal::new(backend).unwrap();
    ///
    /// loop {
    ///     term.draw(|f| {
    ///         let chunks = layout.split(f.size());
    ///         let widget = textarea.widget();
    ///         f.render_widget(widget, chunks[0]);
    ///     }).unwrap();
    ///
    ///     // ...
    /// }
    /// ```
    pub fn widget(&'a self) -> impl Widget + 'a {
        Renderer::new(self)
    }

    /// Set the style of textarea. By default, textarea is not styled.
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// let style = Style::default().fg(Color::Red);
    /// textarea.set_style(style);
    /// assert_eq!(textarea.style(), style);
    /// ```
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Get the current style of textarea.
    pub fn style(&self) -> Style {
        self.style
    }

    /// Set the block of textarea. By default, no block is set.
    /// ```
    /// use tui_textarea::TextArea;
    /// use tui::widgets::{Block, Borders};
    ///
    /// let mut textarea = TextArea::default();
    /// let block = Block::default().borders(Borders::ALL).title("Block Title");
    /// textarea.set_block(block);
    /// assert!(textarea.block().is_some());
    /// ```
    pub fn set_block(&mut self, block: Block<'a>) {
        self.block = Some(block);
    }

    /// Remove the block of textarea which was set by [`TextArea::set_block`].
    /// ```
    /// use tui_textarea::TextArea;
    /// use tui::widgets::{Block, Borders};
    ///
    /// let mut textarea = TextArea::default();
    /// let block = Block::default().borders(Borders::ALL).title("Block Title");
    /// textarea.set_block(block);
    /// textarea.remove_block();
    /// assert!(textarea.block().is_none());
    /// ```
    pub fn remove_block(&mut self) {
        self.block = None;
    }

    /// Get the block of textarea if exists.
    pub fn block<'s>(&'s self) -> Option<&'s Block<'a>> {
        self.block.as_ref()
    }

    /// Set the length of tab character. Due to limitation of tui-rs, hard tab is not supported. Setting 0 disables tab
    /// inputs.
    /// ```
    /// use tui_textarea::{TextArea, Input, Key};
    ///
    /// let mut textarea = TextArea::default();
    /// let tab_input = Input { key: Key::Tab, ctrl: false, alt: false };
    ///
    /// textarea.set_tab_length(8);
    /// textarea.input(tab_input.clone());
    /// assert_eq!(textarea.lines(), ["        "]);
    ///
    /// textarea.set_tab_length(2);
    /// textarea.input(tab_input);
    /// assert_eq!(textarea.lines(), ["          "]);
    /// ```
    pub fn set_tab_length(&mut self, len: u8) {
        self.tab_len = len;
    }

    /// Get how many spaces are used for representing tab character. The default value is 4.
    pub fn tab_length(&self) -> u8 {
        self.tab_len
    }

    /// Set if a hard tab is used or not for indent. When `true` is set, typing a tab key inserts a hard tab instead of
    /// spaces. By default, hard tab is disabled.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.set_hard_tab_indent(true);
    /// textarea.insert_tab();
    /// assert_eq!(textarea.lines(), ["\t"]);
    /// ```
    pub fn set_hard_tab_indent(&mut self, enabled: bool) {
        self.hard_tab_indent = enabled;
    }

    /// Get if a hard tab is used for indent or not.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// assert!(!textarea.hard_tab_indent());
    /// textarea.set_hard_tab_indent(true);
    /// assert!(textarea.hard_tab_indent());
    /// ```
    pub fn hard_tab_indent(&self) -> bool {
        self.hard_tab_indent
    }

    /// Get a string for indent. It consists of spaces by default. When hard tab is enabled, it is a tab character.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// assert_eq!(textarea.indent(), "    ");
    /// textarea.set_tab_length(2);
    /// assert_eq!(textarea.indent(), "  ");
    /// textarea.set_hard_tab_indent(true);
    /// assert_eq!(textarea.indent(), "\t");
    /// ```
    pub fn indent(&self) -> &'static str {
        if self.hard_tab_indent {
            "\t"
        } else {
            spaces(self.tab_len)
        }
    }

    /// Set how many modifications are remembered for undo/redo. Setting 0 disables undo/redo.
    pub fn set_max_histories(&mut self, max: usize) {
        self.history = History::new(max);
    }

    /// Get how many modifications are remembered for undo/redo. The default value is 50.
    pub fn max_histories(&self) -> usize {
        self.history.max_items()
    }

    /// Set the style of line at cursor. By default, the cursor line is styled with underline. To stop styling the
    /// cursor line, set the default style.
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// let style = Style::default().fg(Color::Red);
    /// textarea.set_cursor_line_style(style);
    /// assert_eq!(textarea.cursor_line_style(), style);
    ///
    /// // Disable cursor line style
    /// textarea.set_cursor_line_style(Style::default());
    /// ```
    pub fn set_cursor_line_style(&mut self, style: Style) {
        self.cursor_line_style = style;
    }

    /// Get the style of cursor line. By default it is styled with underline.
    pub fn cursor_line_style(&self) -> Style {
        self.cursor_line_style
    }

    /// Set the style of line number. By setting the style with this method, line numbers are drawn in textarea, meant
    /// that line numbers are disabled by default. If you want to show line numbers but don't want to style them, set
    /// the default style.
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// // Show line numbers in dark gray background
    /// let style = Style::default().bg(Color::DarkGray);
    /// textarea.set_line_number_style(style);
    /// assert_eq!(textarea.line_number_style(), Some(style));
    /// ```
    pub fn set_line_number_style(&mut self, style: Style) {
        self.line_number_style = Some(style);
    }

    /// Remove the style of line number which was set by [`TextArea::set_line_number_style`]. After calling this
    /// method, Line numbers will no longer be shown.
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.set_line_number_style(Style::default().bg(Color::DarkGray));
    /// textarea.remove_line_number();
    /// assert_eq!(textarea.line_number_style(), None);
    /// ```
    pub fn remove_line_number(&mut self) {
        self.line_number_style = None;
    }

    /// Get the style of line number if set.
    pub fn line_number_style(&self) -> Option<Style> {
        self.line_number_style
    }

    /// sets the placeholder text
    pub fn set_placeholder(&mut self, placeholder: String) {
        self.placeholder = Some(placeholder);
    }

    pub fn set_placeholder_style(&mut self, style: Style) {
        self.placeholder_style = Some(style);
    }
    /// Set the style of cursor. By default, a cursor is rendered in the reversed color. Setting the same style as
    /// cursor line hides a cursor.
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// let style = Style::default().bg(Color::Red);
    /// textarea.set_cursor_style(style);
    /// assert_eq!(textarea.cursor_style(), style);
    /// ```
    pub fn set_cursor_style(&mut self, style: Style) {
        self.cursor_style = style;
    }

    /// Get the style of cursor.
    pub fn cursor_style(&self) -> Style {
        self.cursor_style
    }

    /// Get slice of line texts. This method borrows the content, but not moves. Note that the returned slice will
    /// never be empty because an empty text means a slice containing one empty line. This is correct since any text
    /// file must end with a newline.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// assert_eq!(textarea.lines(), [""]);
    ///
    /// textarea.insert_char('a');
    /// assert_eq!(textarea.lines(), ["a"]);
    ///
    /// textarea.insert_newline();
    /// assert_eq!(textarea.lines(), ["a", ""]);
    ///
    /// textarea.insert_char('b');
    /// assert_eq!(textarea.lines(), ["a", "b"]);
    /// ```
    pub fn lines(&'a self) -> &'a [String] {
        &self.lines
    }

    /// Convert [`TextArea`] instance into line texts.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.insert_char('a');
    /// textarea.insert_newline();
    /// textarea.insert_char('b');
    ///
    /// assert_eq!(textarea.into_lines(), ["a", "b"]);
    /// ```
    pub fn into_lines(self) -> Vec<String> {
        self.lines
    }

    /// Get the current cursor position. 0-base character-wise (row, col) cursor position.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    /// assert_eq!(textarea.cursor(), (0, 0));
    ///
    /// textarea.insert_char('a');
    /// textarea.insert_newline();
    /// textarea.insert_char('b');
    ///
    /// assert_eq!(textarea.cursor(), (1, 1));
    /// ```
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Set text alignment. When [`Alignment::Center`] or [`Alignment::Right`] is set, line number is automatically
    /// disabled because those alignments don't work well with line numbers.
    /// ```
    /// use tui_textarea::TextArea;
    /// use tui::layout::Alignment;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.set_alignment(Alignment::Center);
    /// assert_eq!(textarea.alignment(), Alignment::Center);
    /// ```
    pub fn set_alignment(&mut self, alignment: Alignment) {
        if let Alignment::Center | Alignment::Right = alignment {
            self.line_number_style = None;
        }
        self.alignment = alignment;
    }

    /// Get current text alignment. The default alignment is [`Alignment::Left`].
    /// ```
    /// use tui_textarea::TextArea;
    /// use tui::layout::Alignment;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// assert_eq!(textarea.alignment(), Alignment::Left);
    /// ```
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Check if the textarea has a empty content.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let textarea = TextArea::default();
    /// assert!(textarea.is_empty());
    ///
    /// let textarea = TextArea::from(["hello"]);
    /// assert!(!textarea.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.lines == [""]
    }

    /// Get the yanked text. Text is automatically yanked when deleting strings by [`TextArea::delete_line_by_head`],
    /// [`TextArea::delete_line_by_end`], [`TextArea::delete_word`], [`TextArea::delete_next_word`].
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["abc"]);
    ///
    /// textarea.delete_next_word();
    /// assert_eq!(textarea.yank_text(), "abc");
    /// ```
    pub fn yank_text(&'a self) -> &'a str {
        &self.yank
    }

    /// Set a yanked text. The text can be inserted by [`TextArea::paste`]. The string passed to method must not contain
    /// any newlines.
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// textarea.set_yank_text("hello, world");
    /// textarea.paste();
    /// assert_eq!(textarea.lines(), ["hello, world"]);
    /// ```
    pub fn set_yank_text(&mut self, text: impl Into<String>) {
        self.yank = text.into();
    }

    /// Set a regular expression pattern for text search. Setting an empty string stops the text search.
    /// When a valid pattern is set, all matches will be highlighted in the textarea. Note that the cursor does not
    /// move. To move the cursor, use [`TextArea::search_forward`] and [`TextArea::search_back`].
    ///
    /// Grammar of regular expression follows [regex crate](https://docs.rs/regex/latest/regex). Patterns don't match
    /// to newlines so match passes across no newline.
    ///
    /// When the pattern is invalid, the search pattern will not be updated and an error will be returned.
    ///
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["hello, world", "goodbye, world"]);
    ///
    /// // Search "world"
    /// textarea.set_search_pattern("world").unwrap();
    ///
    /// assert_eq!(textarea.cursor(), (0, 0));
    /// textarea.search_forward(false);
    /// assert_eq!(textarea.cursor(), (0, 7));
    /// textarea.search_forward(false);
    /// assert_eq!(textarea.cursor(), (1, 9));
    ///
    /// // Stop the text search
    /// textarea.set_search_pattern("");
    ///
    /// // Invalid search pattern
    /// assert!(textarea.set_search_pattern("(hello").is_err());
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn set_search_pattern(&mut self, query: impl AsRef<str>) -> Result<(), regex::Error> {
        self.search.set_pattern(query.as_ref())
    }

    /// Get a regular expression which was set by [`TextArea::set_search_pattern`]. When no text search is ongoing, this
    /// method returns `None`.
    ///
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// assert!(textarea.search_pattern().is_none());
    /// textarea.set_search_pattern("hello+").unwrap();
    /// assert!(textarea.search_pattern().is_some());
    /// assert_eq!(textarea.search_pattern().unwrap().as_str(), "hello+");
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn search_pattern(&self) -> Option<&regex::Regex> {
        self.search.pat.as_ref()
    }

    /// Search the pattern set by [`TextArea::set_search_pattern`] forward and move the cursor to the next match
    /// position based on the current cursor position. Text search wraps around a text buffer. It returns `true` when
    /// some match was found. Otherwise it returns `false`.
    ///
    /// The `match_cursor` parameter represents if the search matches to the current cursor position or not. When `true`
    /// is set and the cursor position matches to the pattern, the cursor will not move. When `false`, the cursor will
    /// move to the next match ignoring the match at the current position.
    ///
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["hello", "helloo", "hellooo"]);
    ///
    /// textarea.set_search_pattern("hello+").unwrap();
    ///
    /// // Move to next position
    /// let match_found = textarea.search_forward(false);
    /// assert!(match_found);
    /// assert_eq!(textarea.cursor(), (1, 0));
    ///
    /// // Since the cursor position matches to "hello+", it does not move
    /// textarea.search_forward(true);
    /// assert_eq!(textarea.cursor(), (1, 0));
    ///
    /// // When `match_current` parameter is set to `false`, match at the cursor position is ignored
    /// textarea.search_forward(false);
    /// assert_eq!(textarea.cursor(), (2, 0));
    ///
    /// // Text search wrap around the buffer
    /// textarea.search_forward(false);
    /// assert_eq!(textarea.cursor(), (0, 0));
    ///
    /// // `false` is returned when no match was found
    /// textarea.set_search_pattern("bye+").unwrap();
    /// let match_found = textarea.search_forward(false);
    /// assert!(!match_found);
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn search_forward(&mut self, match_cursor: bool) -> bool {
        if let Some(cursor) = self.search.forward(&self.lines, self.cursor, match_cursor) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// Search the pattern set by [`TextArea::set_search_pattern`] backward and move the cursor to the next match
    /// position based on the current cursor position. Text search wraps around a text buffer. It returns `true` when
    /// some match was found. Otherwise it returns `false`.
    ///
    /// The `match_cursor` parameter represents if the search matches to the current cursor position or not. When `true`
    /// is set and the cursor position matches to the pattern, the cursor will not move. When `false`, the cursor will
    /// move to the next match ignoring the match at the current position.
    ///
    /// ```
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::from(["hello", "helloo", "hellooo"]);
    ///
    /// textarea.set_search_pattern("hello+").unwrap();
    ///
    /// // Move to next position with wrapping around the text buffer
    /// let match_found = textarea.search_back(false);
    /// assert!(match_found);
    /// assert_eq!(textarea.cursor(), (2, 0));
    ///
    /// // Since the cursor position matches to "hello+", it does not move
    /// textarea.search_back(true);
    /// assert_eq!(textarea.cursor(), (2, 0));
    ///
    /// // When `match_current` parameter is set to `false`, match at the cursor position is ignored
    /// textarea.search_back(false);
    /// assert_eq!(textarea.cursor(), (1, 0));
    ///
    /// // `false` is returned when no match was found
    /// textarea.set_search_pattern("bye+").unwrap();
    /// let match_found = textarea.search_back(false);
    /// assert!(!match_found);
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn search_back(&mut self, match_cursor: bool) -> bool {
        if let Some(cursor) = self.search.back(&self.lines, self.cursor, match_cursor) {
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// Get the text style at matches of text search. The default style is colored with blue in background.
    ///
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let textarea = TextArea::default();
    ///
    /// assert_eq!(textarea.search_style(), Style::default().bg(Color::Blue));
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn search_style(&self) -> Style {
        self.search.style
    }

    /// Set the text style at matches of text search. The default style is colored with blue in background.
    ///
    /// ```
    /// use tui::style::{Style, Color};
    /// use tui_textarea::TextArea;
    ///
    /// let mut textarea = TextArea::default();
    ///
    /// let red_bg = Style::default().bg(Color::Red);
    /// textarea.set_search_style(red_bg);
    ///
    /// assert_eq!(textarea.search_style(), red_bg);
    /// ```
    #[cfg(feature = "search")]
    #[cfg_attr(docsrs, doc(cfg(feature = "search")))]
    pub fn set_search_style(&mut self, style: Style) {
        self.search.style = style;
    }

    /// Scroll the textarea. See [`Scrolling`] for the argument.
    /// The cursor will not move until it goes out the viewport. When the cursor position is outside the viewport after scroll,
    /// the cursor position will be adjusted to stay in the viewport using the same logic as [`CursorMove::InViewport`].
    ///
    /// ```
    /// # use tui::buffer::Buffer;
    /// # use tui::layout::Rect;
    /// # use tui::widgets::Widget;
    /// use tui_textarea::TextArea;
    ///
    /// // Let's say terminal height is 8.
    ///
    /// // Create textarea with 20 lines "0", "1", "2", "3", ...
    /// let mut textarea: TextArea = (0..20).into_iter().map(|i| i.to_string()).collect();
    /// # // Call `render` at least once to populate terminal size
    /// # let r = Rect { x: 0, y: 0, width: 24, height: 8 };
    /// # let mut b = Buffer::empty(r.clone());
    /// # textarea.widget().render(r, &mut b);
    ///
    /// // Scroll down by 15 lines. Since terminal height is 8, cursor will go out
    /// // the viewport.
    /// textarea.scroll((15, 0));
    /// // So the cursor position was updated to stay in the viewport after the scrolling.
    /// assert_eq!(textarea.cursor(), (15, 0));
    ///
    /// // Scroll up by 5 lines. Since the scroll amount is smaller than the terminal
    /// // height, cursor position will not be updated.
    /// textarea.scroll((-5, 0));
    /// assert_eq!(textarea.cursor(), (15, 0));
    ///
    /// // Scroll up by 5 lines again. The terminal height is 8. So a cursor reaches to
    /// // the top of viewport after scrolling up by 7 lines. Since we have already
    /// // scrolled up by 5 lines, scrolling up by 5 lines again makes the cursor overrun
    /// // the viewport by 5 - 2 = 3 lines. To keep the cursor stay in the viewport, the
    /// // cursor position will be adjusted from line 15 to line 12.
    /// textarea.scroll((-5, 0));
    /// assert_eq!(textarea.cursor(), (12, 0));
    /// ```
    pub fn scroll(&mut self, scrolling: impl Into<Scrolling>) {
        scrolling.into().scroll(&mut self.viewport);
        self.move_cursor(CursorMove::InViewport);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Seaparate tests for ratatui support
    #[test]
    fn scroll() {
        use crate::tui::buffer::Buffer;
        use crate::tui::layout::Rect;
        use crate::tui::widgets::Widget;

        let mut textarea: TextArea = (0..20).into_iter().map(|i| i.to_string()).collect();
        let r = Rect {
            x: 0,
            y: 0,
            width: 24,
            height: 8,
        };
        let mut b = Buffer::empty(r.clone());
        textarea.widget().render(r, &mut b);

        textarea.scroll((15, 0));
        assert_eq!(textarea.cursor(), (15, 0));
        textarea.scroll((-5, 0));
        assert_eq!(textarea.cursor(), (15, 0));
        textarea.scroll((-5, 0));
        assert_eq!(textarea.cursor(), (12, 0));
    }
}
