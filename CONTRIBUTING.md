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
cargo fuzz run edit
```
