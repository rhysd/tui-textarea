<a name="v0.2.0"></a>
# [v0.2.0](https://github.com/rhysd/tui-textarea/releases/tag/v0.2.0) - 18 Oct 2022

- Add [`Scrolling` enum](https://docs.rs/tui-textarea/latest/tui_textarea/enum.Scrolling.html) to provide more flexible scrolling via [`TextArea::scroll`](https://docs.rs/tui-textarea/latest/tui_textarea/struct.TextArea.html#method.scroll) method. It has the following enum variants.
  - **BREAKING** `Scrolling::Delta` scrolls the textarea by given rows and cols. This variant can be converted from `(i16, i16)` so migrating from v0.1.6 is very easy.
    ```rust
    let rows: i16 = ...;
    let cols: i16 = ...;

    // Until v0.1.6
    textarea.scroll(rows, cols);

    // Since v0.2.0
    textarea.scroll((rows, cols));
    ```
  - `Scrolling::PageDown` and `Scrolling::PageUp` scroll the textarea by page.
  - `Scrolling::HalfPageDown` and `Scrolling::HalfPageUp` scroll the textarea by half-page.
- Update default key mappings handled by [`TextArea::input`](https://docs.rs/tui-textarea/latest/tui_textarea/struct.TextArea.html#method.input) method.
  - **BREAKING** Change `PageDown` and `PageUp` keys to scroll down/up the textarea by page since v0.2.0. Until v0.1.6, it moved the cursor down/up by one paragraph.
  - Add `Ctrl+V` and `Alt+V` keys to scroll down/up the textarea by page as Emacs-like key mappings.
  - Add `Alt+]` and `Alt+[` keys to move the cursor down/up by one paragraph as Emacs-like key mappings.
- **BREAKING** Add `#[non_exhaustive]` attribute to [`CursorMove` enum](https://docs.rs/tui-textarea/latest/tui_textarea/enum.CursorMove.html). This is because more cursor move variations may be added in the future.
- Fix panic when the max history size is zero (which means the edit history is disabled). ([#4](https://github.com/rhysd/tui-textarea/issues/4))

[Changes][v0.2.0]


<a name="v0.1.6"></a>
# [v0.1.6](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.6) - 28 Sep 2022

- Support mouse scroll. ([#2](https://github.com/rhysd/tui-textarea/issues/2))
  - Handle mouse events for both `crossterm` and `termion` backends.
  - `TextArea::scroll` method was added.
  - `Key::MouseScrollUp` and `Key::MouseScrollDown` virtual keys are added to `Key` enum so that custom backends can support mouse scrolling.
  - `CursorMove::InViewport` variant was added to `CursorMove` enum, which ensures  the cursor to be within the viewport.
- Add `TextArea::alignment` and `TextArea::set_alignment` to set the text alignment of textarea. Note that right and center alignments don't work well with line number so calling `TextArea::set_alignment` with them automatically disables it. ([#3](https://github.com/rhysd/tui-textarea/issues/3), thanks [@Volkalex28](https://github.com/Volkalex28))
  <img src="https://user-images.githubusercontent.com/823277/192801738-4b9d7a18-e282-4c6c-af73-65a94cd8a721.gif" width=590 height=188>
- Set [`rust-version`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field) to 1.56.1 in `Cargo.toml` to show MSRV explicitly.

[Changes][v0.1.6]


<a name="v0.1.5"></a>
# [v0.1.5](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.5) - 18 Jul 2022

- Improve performance to render a textarea widget. When number of lines increases, now rendering lines is **about 2~8x faster** according to [our benchmark suites](https://github.com/rhysd/tui-textarea/tree/main/bench). See [the commit](https://github.com/rhysd/tui-textarea/commit/4e5b684baf4401337bb2e30fd663fa967321f1c1) for more details of the benchmark results. This was archived by managing a vertical scroll position by ourselves instead of scroll handling by `Paragraph`. Previously, a cost of rendering lines was `O(n)` where `n` was number of all lines. Now the cost is `O(1)`.
- Implement `Clone` for `TextArea` so that textarea instances can be copied easily. It is useful when you create multiple textarea instances with the same configuration. Create a first `TextArea` instance with configuring blocks and styles, then simply clone it.
- Add `arbitrary` feature which is disabled by default. By enabling it, `Input`, `Key` and `CursorMove` can be randomly generated via [arbitrary](https://crates.io/crates/arbitrary) crate. This feature aims to be used by fuzzing tests.
- Add many benchmark suites to track performance; insert/delete lines/characters, text search, moving a cursor.
- Improve fuzzing tests to include rendering a textarea to a dummy terminal backend and moving a cursor randomly.
- Refactor `TextArea` implementation. The implementation of text search was separated to `src/search.rs`. The implementation of highlighting was separated to `src/highlight.rs`. And the implementation of widget rendered by tui-rs was separated to `src/widget.rs`. These refactorings changed no public API.

[Changes][v0.1.5]


<a name="v0.1.4"></a>
# [v0.1.4](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.4) - 10 Jul 2022

- Fix the cursor line style was not applied when a cursor is at the end of line.
- Fix the cursor position after undoing the modification by 'delete until head of line' (`^J` by default).

[Changes][v0.1.4]


<a name="v0.1.3"></a>
# [v0.1.3](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.3) - 08 Jul 2022

- Text search was implemented. Text search is gated behind `search` feature flag to avoid depending on `regex` crate until it is necessary. See [the usage document](https://github.com/rhysd/tui-textarea#text-search-with-regular-expressions), [the API document](https://docs.rs/tui-textarea/latest/tui_textarea/struct.TextArea.html), and [the working example](https://github.com/rhysd/tui-textarea/blob/main/examples/editor.rs) for more details.
  - `TextArea::set_search_pattern` sets a search pattern in regular expression. This updates highlights at matches in textarea, but does not move the cursor.
  - `TextArea::search_forward` moves cursor to the next match of the text search based on current cursor position.
  - `TextArea::search_back` moves cursor to the previous match of the text search based on current cursor position.
  - `TextArea::set_search_style` sets the text style for highlighting matches of text search.
  <img src="https://user-images.githubusercontent.com/823277/177961514-f63e65de-a562-46d9-b858-4c19e55f8772.gif" width=546 height=156 alt="search in editor example">

[Changes][v0.1.3]


<a name="v0.1.2"></a>
# [v0.1.2](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.2) - 25 Jun 2022

- Indent with hard tab is now supported. `TextArea::set_hard_tab_indent` method enables indentation with a hard tab on hitting a tab key.
  ```rust
  let mut textarea = TextArea::default();

  textarea.set_hard_tab_indent(true);
  textarea.insert_tab();
  assert_eq!(textarea.lines(), ["\t"]);
  ```
  Demo with `cargo run --example editor`:
  <img src="https://user-images.githubusercontent.com/823277/175755458-5bf60e84-e01f-410d-9194-d3117031eff6.gif" alt="screencast" width=452 height=140>
- Add `TextArea::indent` method to get an indent string of textarea.
  ```rust
  let mut textarea = TextArea::default();

  assert_eq!(textarea.indent(), "    ");
  textarea.set_tab_length(2);
  assert_eq!(textarea.indent(), "  ");
  textarea.set_hard_tab_indent(true);
  assert_eq!(textarea.indent(), "\t");
  ```

[Changes][v0.1.2]


<a name="v0.1.1"></a>
# [v0.1.1](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.1) - 21 Jun 2022

- Add `TextArea::yank_text` and `TextArea::set_yank_text` to set/get yanked text of the textarea.
  ```rust
  let mut textarea = TextArea::default();
  textarea.set_yank_text("hello, world");
  assert_eq!(textarea.yank_text(), "hello, world");
  textarea.paste();
  assert_eq!(textarea.lines(), ["hello, world"]);
  ```
- Add `CursorMove::Jump(row, col)` variant to move cursor to arbitrary (row, col) position with `TextArea::move_cursor`.
  ```rust
  let mut textarea = TextArea::from(["aaaa", "bbbb"]);
  textarea.move_cursor(CursorMove::Jump(1, 2));
  assert_eq!(textarea.cursor(), (1, 2));
  ```
- Fix hard tabs are not rendered ([#1](https://github.com/rhysd/tui-textarea/issues/1))

[Changes][v0.1.1]


<a name="v0.1.0"></a>
# [v0.1.0](https://github.com/rhysd/tui-textarea/releases/tag/v0.1.0) - 19 Jun 2022

First release :tada:

- GitHub repository: https://github.com/rhysd/tui-textarea
- crates.io: https://crates.io/crates/tui-textarea
- docs.rs: https://docs.rs/tui-textarea/latest/tui_textarea/

[Changes][v0.1.0]


[v0.2.0]: https://github.com/rhysd/tui-textarea/compare/v0.1.6...v0.2.0
[v0.1.6]: https://github.com/rhysd/tui-textarea/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/rhysd/tui-textarea/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/rhysd/tui-textarea/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/rhysd/tui-textarea/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/rhysd/tui-textarea/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/rhysd/tui-textarea/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/rhysd/tui-textarea/tree/v0.1.0

 <!-- Generated by https://github.com/rhysd/changelog-from-release -->
