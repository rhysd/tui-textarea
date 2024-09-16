zeka-tui-textarea
============

Fork of [tui-textarea](https://github.com/rhysd/tui-textarea)

Used for zeka

Small changes to library:

## Full line highlight of cursor line

With this option on, the cursor line will be highlighted for the full width of the textarea.

```rust
textarea.set_cursor_line_fullwidth();
```

## Hide cursor

By default the cursor is hidden when cursor style is the same as cursor_line style. But cursor is always drawed.

With this option on, cursor is not drawed at all.

It allows inclusive selection to be displayed properly.

```rust
textarea.set_cursor_hidden();
```
