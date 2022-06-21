## Reporting an issue

For reporting a bug, please make sure your report includes the following points.

- How to reproduce it
  - What text the textarea contained as pre-condition
  - What operations you did
  - What was the expected behavior
  - What was the actual behavior
- Environment
  - Your terminal
  - Rust version
  - `tui` crate version

An example of bug report: https://github.com/rhysd/tui-textarea/issues/1

## Submitting a pull request

Please ensure that all tests and linter checks passed on your branch before creating a PR which modifies some source files.

To run tests:

```sh
cargo test -- --skip src/lib.rs
```

`--skip` is necessary since `cargo test` tries to run code blocks in [README file](./README.md).

To run linters:

```sh
cargo clippy --all-features --examples
cargo fmt -- --check
```

To run fuzzer:

[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) is necessary.

```sh
cargo +nightly fuzz run edit
```
