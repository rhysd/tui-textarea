use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tui_textarea::{Input, Key, TextArea};
use tui_textarea_bench::dummy_terminal;

const LOREM: &[&str] = &[
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do",
    "eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim",
    "ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut",
    "aliquip ex ea commodo consequat. Duis aute irure dolor in",
    "reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla",
    "pariatur. Excepteur sint occaecat cupidatat non proident, sunt in",
    "culpa qui officia deserunt mollit anim id est laborum.",
];

#[inline]
fn append_lorem(repeat: usize) -> usize {
    let mut textarea = TextArea::default();
    let mut term = dummy_terminal();
    for _ in 0..repeat {
        for line in LOREM {
            for c in line.chars() {
                textarea.input(Input {
                    key: Key::Char(c),
                    ctrl: false,
                    alt: false,
                });
            }
            term.draw(|f| {
                f.render_widget(textarea.widget(), f.size());
            })
            .unwrap();
        }
        textarea.input(Input {
            key: Key::Enter,
            ctrl: false,
            alt: false,
        });
        term.draw(|f| {
            f.render_widget(textarea.widget(), f.size());
        })
        .unwrap();
    }
    textarea.lines().len()
}

fn append(c: &mut Criterion) {
    c.bench_function("append::1_lorem", |b| b.iter(|| black_box(append_lorem(1))));
    c.bench_function("append::10_lorem", |b| {
        b.iter(|| black_box(append_lorem(10)))
    });
    c.bench_function("append::100_lorem", |b| {
        b.iter(|| black_box(append_lorem(100)))
    });
}

criterion_group!(insert, append);
criterion_main!(insert);
