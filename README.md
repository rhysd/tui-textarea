tui-textarea
============

[tui-textarea][crate] is a simple yet powerful text editor widget like `<textarea>` in HTML for [tui-rs][]. Multi-line
text editor can be easily put as part of your TUI application.

**Features:**

- Multi-line text editor widget with basic operations (insert/delete characters, auto scrolling, ...)
- Emacs-like shortcuts (`C-n`/`C-p`/`C-f`/`C-b`, `M-f`/`M-b`, `C-a`/`C-e`, `C-h`/`C-d`, `C-k`, `M-<`/`M->`, ...)
- Undo/Redo
- Line number
- Cursor line highlight
- Yank support. Paste text deleted with `C-k`, `C-j`, ...
- Backend agnostic. [crossterm][], [termion][], and your own backend are all supported
- Multiple textarea widgets in the same screen

[Documentation][doc]

TODO: Screenshot

## Installation

Add `tui-textarea` crate to dependencies in your `Cargo.toml`. This enables crossterm backend support by default.

```toml
[dependencies]
tui = "*"
tui-textarea = "*"
```

If you're using tui-rs with [termion][], enable `termion` feature instead of `crossterm` feature.

```toml
[dependencies]
tui = { version = "*", default-features=false, features=["termion"] }
tui-textarea = { version = "*", default-features=false, features=["termion"] }
```

## Examples

Running `cargo run` in this repository can demonstrate usage of tui-textarea.

```sh
cargo run --example minimal
```

**Examples:**

- [minimal](./examples/minimal.rs): Minimal usage with [crossterm][] support
- [termion](./examples/termion.rs): Minimal usage with [termion][] support

## Minimal Usage

```rust
use tui_textarea::TextArea;
use crossterm::event::{Event, read};

let mut term = tui::Terminal::new(...);

// Create an empty `TextArea` instance which manages the editor state
let mut textarea = TextArea::default();

// Event loop
loop {
    term.draw(|f| {
        // Get `tui::layout::Rect` where the editor should be rendered
        let rect = ...;
        // `TextArea::widget` builds a widget to render the editor with tui
        let widget = textarea.widget();
        // Render the widget in terminal screen
        f.render_widget(widget, rect);
    })?;

    if let Event::Key(key) = read()? {
        // Your own key mapping to break the event loop
        if key.code == KeyCode::Esc {
            break;
        }
        // `TextArea::input` can directly handle key events from backends and update the editor state
        textarea.input(key);
    }
}

// Get text lines as `&[String]`
println!("Lines: {:?}", textarea.lines());
```

`TextArea` is an instance to manage the editor state. By default, it disables line numbers and highlights cursor line
with underline.

`TextArea::widget()` builds a widget to render the current state of the editor. Create the widget and render it on each
tick of event loop.

`TextArea::input()` receives inputs from tui backends. The method can take key events from backends such as
`crossterm::event::KeyEvent` or `termion::event::Key` directly if the features are enabled. The method handles default
key mappings as well.

Default key mappings are as follows:

| Mappings                                     | Description                               |
|----------------------------------------------|-------------------------------------------|
| `Ctrl+H`, `Backspace`                        | Delete one character before cursor        |
| `Ctrl+D`, `Delete`                           | Delete one character next to cursor       |
| `Ctrl+M`, `Enter`                            | Insert newline                            |
| `Ctrl+K`                                     | Delete from cursor until the end of line  |
| `Ctrl+J`                                     | Delete from cursor until the head of line |
| `Ctrl+W`, `Alt+H`, `Alt+Backspace`           | Delete one word before cursor             |
| `Alt+D`, `Alt+Delete`                        | Delete one word next to cursor            |
| `Ctrl+U`                                     | Undo                                      |
| `Ctrl+R`                                     | Redo                                      |
| `Ctrl+Y`, `Ctrl+V`                           | Paste yanked text                         |
| `Ctrl+F`, `→`                                | Move cursor forward by one character      |
| `Ctrl+B`, `←`                                | Move cursor backward by one character     |
| `Ctrl+P`, `↑`                                | Move cursor up by one line                |
| `Ctrl+N`, `↓`                                | Move cursor down by one line              |
| `Alt+F`, `Ctrl+→`                            | Move cursor forward by word               |
| `Atl+B`, `Ctrl+←`                            | Move cursor backward by word              |
| `Alt+P`, `Ctrl+↑`                            | Move cursor up by paragraph               |
| `Alt+N`, `Ctrl+↓`                            | Move cursor down by paragraph             |
| `Ctrl+E`, `End`, `Ctrl+Alt+F`, `Ctrl+Alt+→`  | Move cursor to the end of line            |
| `Ctrl+A`, `Home`, `Ctrl+Alt+B`, `Ctrl+Alt+←` | Move cursor to the head of line           |
| `Alt+<`, `Ctrl+Alt+P`, `Ctrl+Alt+↑`          | Move cursor to top of lines               |
| `Alt+>`, `Ctrl+Alt+N`, `Ctrl+Alt+↓`          | Move cursor to bottom of lines            |

Deleting multiple characters at once saves the deleted text to yank buffer. It can be pasted with `Ctrl+Y` or `Ctrl+V`
later.

If you don't want to use default key mappings, see the 'Advanced Usage' section.

## Basic Usage

### Create `TextArea` instance with text

`TextArea` implements `Default` trait to create an editor instance with an empty text.

```rust
let mut textarea = TextArea::default();
```

`TextArea::new()` creates an editor instance with text lines passed as `Vec<String>`.

```rust
let mut lines: Vec<String> = ...;
let mut textarea = TextArea::new(lines);
```

`TextArea` implements `From<impl Iterator<Item=impl Into<String>>>`. `TextArea::from()` can create an editor instance
from any iterators whose elements can be converted to `String`.

```rust
// Create `TextArea` from from `[&str]`
let mut textarea = TextArea::from([
    "this is first line",
    "this is second line",
    "this is third line",
]);

// Create `TextArea` from `String`
let mut text: String = ...;
let mut textarea = TextARea::from(text.lines());
```

`TextArea` also implements `FromIterator<impl Into<String>>`. `Iterator::collect()` can collect strings as an editor
instance. This allows to create `TextArea` reading lines from file efficiently using `io::BufReader`.

```rust
let file = fs::File::open(path)?;
let mut textarea: TextArea = io::BufReader::new(file).lines().collect::<io::Result<_>>()?;
```

### Get text contents from `TextArea`

`TextArea::lines()` returns text lines as `&[String]`. It borrows text contents temporarily.

```rust
let text: String = textarea.lines().join("\n");
```

`TextArea::into_lines()` moves `TextArea` instance into text lines as `Vec<String>`. This can retrieve the text contents
without any copy.

```rust
let lines: Vec<String> = textarea.into_lines();
```

Note that `TextArea` always contains at least one line. For example, an empty text means one empty line. This is because
any text file must end with newline.

```rust
let textarea = TextArea::default();
assert_eq!(textarea.into_lines(), [""]);
```

### Show line number

By default, `TextArea` does now show line numbers. To enable, set a style for rendering line numbers by
`TextArea::set_line_number_style()`. For example, the following renders line numbers in dark gray background
color.

```rust
use tui::style::{Style, Color};

let style = Style::default().bg(Color::DarkGray);
textarea.set_line_number_style(style);
```

### Configure cursor line style

By default, `TextArea` renders the line at cursor with underline so that users can easily notice where the current line
is. To change the style of cursor line, use `TextArea::set_cursor_line_style()`. For example, the following styles the
cursor line with bold text.

```rust
use tui::style::{Style, Modifier};

let style = Style::default().add_modifier(Modifier::BOLD);
textarea.set_line_number_style(style);
```

To disable cursor line style, set the default style as follows:

```rust
use tui::style::{Style, Modifier};

textarea.set_line_number_style(Style::default());
```

### Configure tab width

The default tab width is 4. To change it, use `TextArea::set_tab_length()` method. The following sets 2 to tab width.
Typing tab key inserts 2 spaces.

```rust
textarea.set_tab_length(2);
```

### Configure max history size

By default, past 50 modifications are stored as edit history. The history is used for undo/redo. To change how many past
edits are remembered, use `TextArea::set_max_histories()` method. The following remembers past 1000 changes.

```rust
textarea.set_max_histories(1000);
```

Setting 0 disables undo/redo.

```rust
textarea.set_max_histories(0);
```

## Advanced Usage

### Single line input like `<input>` in HTML

To use `TextArea` for single line input widget like `<input>` in HTML, ignore all key mappings which inserts newline.

```rust
use crossterm::event::{Event, read};
use tui_textarea::{Input, Key};

let default_text: &str = ...;
let default_text = default_text.replace(&['\n', '\r'], " "); // Ensure no new line is contained
let mut textarea = TextArea::new(vec![default_text]);

// Event loop
loop {
    // ...

    // Using `Input` is not mandatory, but it's useful for pattern match
    // Ignore Ctrl+m and Enter. Otherwise handle keys as usual
    match Input::from(read()?) {
        Input { key: Key::Char('m'), ctrl: true, alt: false }
        | Input { key: Key::Enter, .. } => continue,
        input => textarea.input(key),
    }
}

let text = textarea.into_lines().remove(0); // Get input text
```

### Define your own key mappings

All editor operations are defined as public methods of `TextArea`. To move cursor, use `tui_textarea::CursorMove` to
notify how to move the cursor.

| Method                                               | Operation                                 |
|------------------------------------------------------|-------------------------------------------|
| `textarea.delete_char()`                             | Delete one character before cursor        |
| `textarea.delete_next_char()`                        | Delete one character next to cursor       |
| `textarea.insert_newline()`                          | Insert newline                            |
| `textarea.delete_line_by_end()`                      | Delete from cursor until the end of line  |
| `textarea.delete_line_by_head()`                     | Delete from cursor until the head of line |
| `textarea.delete_word()`                             | Delete one word before cursor             |
| `textarea.delete_next_word()`                        | Delete one word next to cursor            |
| `textarea.undo()`                                    | Undo                                      |
| `textarea.redo()`                                    | Redo                                      |
| `textarea.paste()`                                   | Paste yanked text                         |
| `textarea.move_cursor(CursorMove::Forward)`          | Move cursor forward by one character      |
| `textarea.move_cursor(CursorMove::Back)`             | Move cursor backward by one character     |
| `textarea.move_cursor(CursorMove::Up)`               | Move cursor up by one line                |
| `textarea.move_cursor(CursorMove::Down)`             | Move cursor down by one line              |
| `textarea.move_cursor(CursorMove::WordForward)`      | Move cursor forward by word               |
| `textarea.move_cursor(CursorMove::WordBack)`         | Move cursor backward by word              |
| `textarea.move_cursor(CursorMove::ParagraphForward)` | Move cursor up by paragraph               |
| `textarea.move_cursor(CursorMove::ParagraphBack)`    | Move cursor down by paragraph             |
| `textarea.move_cursor(CursorMove::End)`              | Move cursor to the end of line            |
| `textarea.move_cursor(CursorMove::Head)`             | Move cursor to the head of line           |
| `textarea.move_cursor(CursorMove::Top)`              | Move cursor to top of lines               |
| `textarea.move_cursor(CursorMove::Bottom)`           | Move cursor to bottom of lines            |

To define your own key mappings, simply call the above methods in your code instead of `TextArea::input()` method. The
following example defines modal key mappings like Vim.

```rust
use crossterm::event::{Event, read};
use tui_textarea::{Input, Key, CursorMove};

let mut textarea = ...;

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    Insert,
}

let mut mode = Mode::Normal;

// Event loop
loop {
    // ...

    match mode {
        Mode::Normal => match Input::from(read()?) {
            Input { key: Key::Char('h'), .. } => textarea.move_cursor(CursorMove::Back),
            Input { key: Key::Char('j'), .. } => textarea.move_cursor(CursorMove::Down),
            Input { key: Key::Char('k'), .. } => textarea.move_cursor(CursorMove::Up),
            Input { key: Key::Char('l'), .. } => textarea.move_cursor(CursorMove::Forward),
            Input { key: Key::Char('w'), .. } => textarea.move_cursor(CursorMove::WordForward),
            Input { key: Key::Char('b'), .. } => textarea.move_cursor(CursorMove::WordBack),
            Input { key: Key::Char('D'), .. } => textarea.delete_line_by_end(),
            Input { key: Key::Char('u'), .. } => textarea.undo(),
            Input { key: Key::Char('R'), ctrl: true, .. } => textarea.redo(),
            Input { key: Key::Char('i'), .. } => mode = Mode::Insert,
            _ => {},
        },
        Mode::Insert => match Input::from(read()?) {
            Input { key: Key::Esc, .. } => mode = Mode::Normal,
            input => textarea.input(input), // Use default key mappings in insert mode
        },
    }
}
```

If you don't want to use default key mappings, `TextArea::input_without_shortcuts()` method can be used instead of
`TextArea::input()`. The method only handles very basic operations such as inserting/deleting single characters, tabs,
newlines.

```rust
match Input::from(read()?) {
    // Handle your own key mappings here
    // ...
    input => textarea.input_without_shortcuts(input),
}
```

### Use your own backend

tui-rs allows to make your own backend by implementing [`tui::backend::Backend`][tui-backend] trait. tui-textarea also
supports it. In the case, support for neither crossterm nor termion is necessary.

```toml
[dependencies]
tui = { version = "*", default-features=false }
tui-textarea = { version = "*", default-features=false }
```

`tui_textarea::Input` is a type for backend-agnostic key input. What you need to do is converting key event in your own
backend into the `tui_textarea::Input` instance. Then `TextArea::input()` method can handle the input as other backend.

In the following example, let's say `your_backend::KeyDown` is a key event type for your backend and
`your_backend::read_next_key()` returns the next key event.

```rust
// In your backend implementation

pub enum KeyDown {
    Char(char),
    BS,
    Del,
    Esc,
    // ...
}

// Return tuple of (key, ctrlkey, altkey)
pub fn read_next_key() -> (KeyDown, bool, bool) {
    // ...
}
```

Then you can implement the logic to convert `your_backend::KeyDown` value into `tui_textarea::Input` value.

```rust
use tui_textarea::{Input, Key};
use your_backend::KeyDown;

fn keydown_to_input(key: KeyDown, ctrl: bool, alt: bool) -> Input {
    match key {
        KeyDown::Char(c) => Input { key: Key::Char(c), ctrl, alt },
        KeyDown::BS => Input { key: Key::Backspace, ctrl, alt },
        KeyDown::Del => Input { key: Key::Delete, ctrl, alt },
        KeyDown::Esc => Input { key: Key::Esc, ctrl, alt },
        // ...
        _ => Input::default(),
    }
}
```

For the keys which are not handled by tui-textarea, `tui_textarea::Input::default()` is available. It returns 'null'
key. An editor will do nothing with the key.

Finally, convert your own backend's key input type into `tui_textarea::Input` and pass it to `TextArea::input()`.

```rust
let mut textarea = ...;

// Event loop
loop {
    // ...

    let (key, ctrl, alt) = your_backend::read_next_key();
    if key == your_backend::KeyDown::Esc {
        break; // For example, quit your app on pressing Esc
    }
    textarea.input(keydown_to_input(key, ctrl, alt));
}
```

### Put multiple `TextArea` instances in screen

You don't need to do anything special. Create multiple `TextArea` instances and render widgets built from each instances.

The following is an example to put two textarea widgets in application and manage the focus.

```rust
use tui_textarea::{TextArea, Input, Key};
use crossterm::event::{Event, read};

let editors = &mut [
    TextArea::default(),
    TextArea::default(),
];

let mut focused = 0;

loop {
    term.draw(|f| {
        let rects = ...;

        for (editor, rect) in editors.iter_mut().zip(rects.into_iter()) {
            let widget = editor.widget();
            f.render_widget(widget, rect);
        }
    })?;

    match Input::from(read()?) {
        // Switch focused textarea by Ctrl+S
        Input { key: Key::Char('s'), ctrl: true, .. } => focused = (focused + 1) % 2;
        // Handle input by the focused editor
        input => editors[focused].input(input),
    }
}
```

## Contributing to tui-textarea

This project is developed [on GitHub][repo].

For feature requests or bug reports, please [create an issue][new-issue]. For submitting patches, please [create a pull
request][pulls].

Please see [CONTRIBUTING.md](./CONTRIBUTING.md) before making a PR.

## License

tui-textarea is distributed under [The MIT License](./LICENSE.txt).

[crate]: https://crates.io/crates/tui-textarea
[tui-rs]: https://github.com/fdehau/tui-rs
[termion]: https://docs.rs/termion/latest/termion/
[crosster]: https://docs.rs/crossterm/latest/crossterm/
[doc]: https://docs.rs/tui-textarea/latest/tui_textarea
[tui-backend]: https://docs.rs/tui/latest/tui/backend/trait.Backend.html
[repo]: https://github.com/rhysd/tui-textarea
[new-issue]: https://github.com/rhysd/tui-textarea/issues/new
[pulls]: https://github.com/rhysd/tui-textarea/pulls
