use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tui_textarea::{CursorMove, Input, Key, TextArea};
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
const SEED: [u8; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
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
                term.draw(|f| {
                    f.render_widget(textarea.widget(), f.size());
                })
                .unwrap();
            }
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

#[inline]
fn random_lorem(repeat: usize) -> usize {
    let mut rng = SmallRng::from_seed(SEED);
    let mut textarea = TextArea::default();
    let mut term = dummy_terminal();

    for _ in 0..repeat {
        for line in LOREM {
            let row = rng.gen_range(0..textarea.lines().len() as u16);
            textarea.move_cursor(CursorMove::Jump(row, 0));
            textarea.move_cursor(CursorMove::End);

            textarea.input(Input {
                key: Key::Enter,
                ctrl: false,
                alt: false,
            });
            term.draw(|f| {
                f.render_widget(textarea.widget(), f.size());
            })
            .unwrap();

            for c in line.chars() {
                textarea.input(Input {
                    key: Key::Char(c),
                    ctrl: false,
                    alt: false,
                });
                term.draw(|f| {
                    f.render_widget(textarea.widget(), f.size());
                })
                .unwrap();
            }
        }
    }

    textarea.lines().len()
}

fn append(c: &mut Criterion) {
    c.bench_function("append::1_lorem", |b| b.iter(|| black_box(append_lorem(1))));
    c.bench_function("append::10_lorem", |b| {
        b.iter(|| black_box(append_lorem(10)))
    });
    c.bench_function("append::50_lorem", |b| {
        b.iter(|| black_box(append_lorem(50)))
    });
}

fn random(c: &mut Criterion) {
    c.bench_function("random::1_lorem", |b| b.iter(|| black_box(random_lorem(1))));
    c.bench_function("random::10_lorem", |b| {
        b.iter(|| black_box(random_lorem(10)))
    });
    c.bench_function("random::50_lorem", |b| {
        b.iter(|| black_box(random_lorem(50)))
    });
}

criterion_group!(insert, append, random);
criterion_main!(insert);
