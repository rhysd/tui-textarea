zeka-tui-textarea
============

Fork of [tui-textarea](https://github.com/rhysd/tui-textarea)

Used for zeka. Please use original library.

- Needs unit tests and tests
- Needs optimization
- Needs refactoring

Changes to library:

## Full line highlight of cursor line

With this option on, the cursor line will be highlighted for the full width of the textarea.

```rust
use tui_textarea::TextArea;
let mut textarea = TextArea::default();
textarea.set_cursor_line_fullwidth();
```

## Hide cursor

By default the cursor is hidden when cursor style is the same as cursor_line style. But cursor is always drawed.

With this option on, cursor is not drawed at all.

It allows inclusive selection to be displayed properly.

```rust
use tui_textarea::TextArea;
let mut textarea = TextArea::default();
textarea.set_cursor_hidden();
```

## Inclusive selection

With this option on, the selection includes the char under cursor. Used eg for vim mode.

```rust
use tui_textarea::TextArea;
let mut textarea = TextArea::default();
textarea.set_selection_inclusive();
```

## Additional `CursorMove` movements

- `HeadNonSpace`: Move cursor to the first non space character of line.
- `WordSpacingForward`: Move cursor forward by one WORD. Word boundary appears at spaces.
- `WordSpacingEnd`: Move cursor forward to the next end of WORD. WORD boundary appears at spaces.
- `WordSpacingBack`: Move cursor backward by one WORD. WORD boundary appears at spaces.

## TextWrap

Adds three text wraping modes

- `TextWrapMode::Width` => text wrap at textarea width without looking at words
- `TextWrapMode::Word` => text wrap at words separated by whitespace or punctuations
- `TextWrapMode::WORD` => text wrap at words separated by whitespace only

```rust
use tui_textarea::TextArea;
use tui_textarea::TextWrapMode;
let mut textarea = TextArea::default();
textarea.set_textwrap(TextWrapMode::Word);
```

Note that this is not optimized and uses a simple word wraping algorithm.

**IMPORTANT**: Note also that I changed `CursorMove::Top`, `CursorMove::Bottom`, `CursorMove::ParagraphForward`, `CursorMove::ParagraphBack` to move cursor to start or end of line.

## Helix light example

Beeing an fan of [helix editor](https://helix-editor.com/), I tried to replicate the main commands from helix keymaps, as well as the overall look. This is not a perfect equivalent but looks OK to me.

- movements
- changes and inserts
- selections
- minor goto mode

See file `examples/helix_light.rs` for details.
