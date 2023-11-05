<a name="v0.3.1"></a>
# [v0.3.1](https://github.com/rhysd/tui-textarea/releases/tag/v0.3.1) - 04 Nov 2023

- Fix the width of rendered tab character (`\t`) is wrong in some cases when hard tab is enabled by `TextArea::set_hard_tab_indent` ([#43](https://github.com/rhysd/tui-textarea/issues/43)).
- Fix key inputs are doubled on Windows when converting from `crossterm::event::KeyEvent` into `tui_textarea::Input`. Note that the conversion from `crossterm::event::Event` into `tui_textarea::Input` does not have this issue.
- Support converting the following type instances into `tui_textarea::Input`.
  - `crossterm::event::KeyCode`
  - `crossterm::event::KeyEvent`
  - `crossterm::event::MouseEvent`
  - `crossterm::event::MouseKind`
  - `termwiz::input::KeyCode`
  - `termwiz::input::KeyEvent`
  - `termion::event::MouseButton`
- Fix typos in API document and error message ([#40](https://github.com/rhysd/tui-textarea/issues/40), thanks [@fritzrehde](https://github.com/fritzrehde)).

[Changes][v0.3.1]


<a name="v0.3.0"></a>
# [v0.3.0](https://github.com/rhysd/tui-textarea/releases/tag/v0.3.0) - 24 Oct 2023

- **BREAKING CHANGE:** Enable ratatui support by default instead of inactive tui-rs.
  - `ratatui-` prefix is removed from all `ratatui-*` features. `crossterm`, `termion`, and `termwiz` features are for ratatui:
    ```yaml
    # ratatui with crossterm backend
    tui-textarea = "0.3"
    # ratatui with termwiz backend
    tui-textarea = { version = "0.3", features = ["termwiz"], default-features = false }
    # ratatui with termion backend
    tui-textarea = { version = "0.3", features = ["termion"], default-features = false }
    ```
  - Instead, features for tui-rs support are now prefixed with `tuirs-`:
    ```yaml
    # tui-rs with crossterm backend
    tui-textarea = { version = "0.3", features = ["tuirs-crossterm"], default-features = false }
    # Use proper version of crossterm
    crossterm = "0.2.5"
    ```
  - Examples and documents are now implemented and described with ratatui by default
- **BREAKING CHANGE:** Rename `your-backend` features to `no-backend`. You need to update the feature names if you're using tui-textarea with your own backend.
- Relax the restriction of ratatui crate dependency from `0.23.0` to `>=0.23.0, <1`, which means 'v0.23.0 or later and earlier than v1'. The latest version of ratatui (v0.24.0) now works with tui-textarea ([#36](https://github.com/rhysd/tui-textarea/issues/36)).
- Enable `termwiz` and `termion` features on generating the API document. APIs to convert from input events of termwiz/termion to [`tui_textarea::Input`](https://docs.rs/tui-textarea/latest/tui_textarea/struct.Input.html) are now listed in the document.

Previous Backend features table (v0.2.4):

|         | crossterm                        | termion           | termwiz           | Your own backend       |
|---------|----------------------------------|-------------------|-------------------|------------------------|
| tui-rs  | `crossterm` (enabled by default) | `termion`         | N/A               | `your-backend`         |
| ratatui | `ratatui-crossterm`              | `ratatui-termion` | `ratatui-termwiz` | `ratatui-your-backend` |

New backend features table (v0.3.0):

|         | crossterm                        | termion         | termwiz   | Your own backend   |
|---------|----------------------------------|-----------------|-----------|--------------------|
| tui-rs  | `tuirs-crossterm`                | `tuirs-termion` | N/A       | `tuirs-no-backend` |
| ratatui | `crossterm` (enabled by default) | `termion`       | `termwiz` | `no-backend`       |


[Changes][v0.3.0]


<a name="v0.2.4"></a>
# [v0.2.4](https://github.com/rhysd/tui-textarea/releases/tag/v0.2.4) - 21 Oct 2023

- Support the ratatui's [termwiz](https://crates.io/crates/termwiz) backend. `ratatui-termwiz` feature was newly added for this.
  - Add the following dependencies in your Cargo.toml to use termwiz support.
    ```toml
    termwiz = "0.20"
    ratatui = { version = "0.23", default-features = false, features = ["termwiz"] }
    tui-textarea = { version = "0.2.4", default-features = false, features = ["ratatui-termwiz"] }
    ```
  - Read and run [the `termwiz` example](https://github.com/rhysd/tui-textarea/blob/main/examples/termwiz.rs) to know the API usage.
    ```sh
    cargo run --example termwiz --no-default-features --features=ratatui-termwiz
    ```
- Fix calculating the length of tab character when the line contains wide characters. Now the length of wide characters like あ are calculated as 2 correctly.

[Changes][v0.2.4]


<a name="v0.2.3"></a>
# [v0.2.3](https://github.com/rhysd/tui-textarea/releases/tag/v0.2.3) - 20 Oct 2023

- Add APIs to mask text with a character ([#32](https://github.com/rhysd/tui-textarea/issues/32), thanks [@pm100](https://github.com/pm100)).
  - `TextArea::set_mask_char`, `TextArea::clear_mask_char`, `TextArea::mask_char` are added. See [the documentation](https://docs.rs/tui-textarea/latest/tui_textarea/) for more details.
  - The [`password`](https://github.com/rhysd/tui-textarea/blob/main/examples/password.rs) example was added to show the usage.
    <img src="https://raw.githubusercontent.com/rhysd/ss/master/tui-textarea/password.gif" width=589 height=92 alt="password example">
- Fix the length of displayed hard tab in text ([#33](https://github.com/rhysd/tui-textarea/issues/33), thanks [@pm100](https://github.com/pm100)).

[Changes][v0.2.3]


<a name="v0.2.2"></a>
# [v0.2.2](https://github.com/rhysd/tui-textarea/releases/tag/v0.2.2) - 01 Oct 2023

Very small patch release only for fixing [the build failure on docs.rs](https://docs.rs/crate/tui-textarea/0.2.1/builds/926847). No implementation has been changed.

[Changes][v0.2.2]


<a name="v0.2.1"></a>
# [v0.2.1](https://github.com/rhysd/tui-textarea/releases/tag/v0.2.1) - 01 Oct 2023

- Add the support for [ratatui](https://crates.io/crates/ratatui) crate in addition to [tui-rs](https://crates.io/crates/tui). The ratatui crate is a community fork of inactive tui-rs crate. ([#12](https://github.com/rhysd/tui-textarea/issues/12))
  - The latest version of ratatui v0.23 is supported.
  - tui-textarea still uses tui-rs by default to keep the compatibility at this moment. ratatui users explicitly need to set features for it. See [the installation document](https://github.com/rhysd/tui-textarea#installation) for the features matrix. For example, when you want to use ratatui and crossterm, write the following in your `Cargo.toml`:
    ```toml
    [dependencies]
    ratatui = "*"
    tui-textarea = { version = "*", features = ["ratatui-crossterm"], default-features = false }
    ```
  - tui-rs is no longer maintained and the repository was archived. At the next minor version bump, tui-textarea will switch the default features from tui-rs to ratatui. If you use tui-rs, I recommend to switch your dependency to ratatui.
  - Examples with ratatui are added to [the `examples` directory](https://github.com/rhysd/tui-textarea/tree/main/examples). For example, the following command runs ratatui version of `editor` example:
    ```sh
    cargo run --example ratatui_editor --no-default-features --features=ratatui-crossterm,search file.txt
    ```
- Add support for the placeholder text which is rendered when no text is input in the textarea. ([#16](https://github.com/rhysd/tui-textarea/issues/16), thanks [@pm100](https://github.com/pm100))
  - Use `TextArea::set_placeholder_text` to set the text. To change the text style, use `TextArea::set_placeholder_style`. See [the API documentation](https://docs.rs/tui-textarea/latest/tui_textarea/struct.TextArea.html) for more details.
  - `popup_placeholder` example was added to show the usage.
    ```sh
    cargo run --example popup_placeholder
    ```
- Derive `Debug` trait for `TextArea` struct. ([#23](https://github.com/rhysd/tui-textarea/issues/23))
- Fix a key input is received twice on Windows. ([#17](https://github.com/rhysd/tui-textarea/issues/17), thanks [@pm100](https://github.com/pm100))

[Changes][v0.2.1]


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


[v0.3.1]: https://github.com/rhysd/tui-textarea/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rhysd/tui-textarea/compare/v0.2.4...v0.3.0
[v0.2.4]: https://github.com/rhysd/tui-textarea/compare/v0.2.3...v0.2.4
[v0.2.3]: https://github.com/rhysd/tui-textarea/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/rhysd/tui-textarea/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/rhysd/tui-textarea/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rhysd/tui-textarea/compare/v0.1.6...v0.2.0
[v0.1.6]: https://github.com/rhysd/tui-textarea/compare/v0.1.5...v0.1.6
[v0.1.5]: https://github.com/rhysd/tui-textarea/compare/v0.1.4...v0.1.5
[v0.1.4]: https://github.com/rhysd/tui-textarea/compare/v0.1.3...v0.1.4
[v0.1.3]: https://github.com/rhysd/tui-textarea/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/rhysd/tui-textarea/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/rhysd/tui-textarea/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/rhysd/tui-textarea/tree/v0.1.0

<!-- Generated by https://github.com/rhysd/changelog-from-release v3.7.0 -->
