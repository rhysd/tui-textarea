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
cargo test --features=search -- --skip src/lib.rs
```

`--skip` is necessary since `cargo test` tries to run code blocks in [README file](./README.md).

To run linters:

```sh
cargo clippy --all-features --examples
cargo fmt -- --check
```

If you use [cargo-watch][], `cargo watch-check` alias is useful to run checks automatically on writing to a file.

## Print debug

Since this crate uses stdout, `println!` is not available for debugging. Instead, stderr through [`eprintln!`][eprintln]
or [`dbg!`][dbg] are useful.

At first, add prints where you want to debug:

```rust
eprintln!("some value is {:?}", some_value);
dbg!(&some_value);
```

Then redirect stderr to some file:

```sh
cargo run -- --example minimal 2>debug.txt
```

Then the debug prints are output to the `debug.txt` file. If timing is important or you want to see the output in real-time,
printing the file content with `tail` command would be useful.

```sh
# In a terminal, reproduce the issue
cargo run -- --example minimal 2>debug.txt

# In another terminal, run `tail` command to monitor the content
tail -F debug.txt
```

## Running a fuzzer

To run fuzzing tests, [cargo-fuzz][] is necessary.

```sh
cargo +nightly fuzz run edit
```

## Running benchmark suites

Benchmarks are available using [Criterion.rs][criterion].

To separate `criterion` crate dependency, benchmark suites are separated as another crate in [bench/](./bench).

To run benchmarks:

```sh
cd ./bench
cargo bench --benches
```

See [README in bench/](./bench/README.md) for more details.

[cargo-watch]: https://crates.io/crates/cargo-watch
[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz
[criterion]: https://github.com/bheisler/criterion.rs
[eprintln]: https://doc.rust-lang.org/std/macro.eprintln.html
[dbg]: https://doc.rust-lang.org/std/macro.dbg.html
