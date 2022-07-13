use criterion::{criterion_group, criterion_main, Criterion};
use tui_textarea::{CursorMove, TextArea};
use tui_textarea_bench::{dummy_terminal, TerminalExt, LOREM};

#[derive(Clone, Copy)]
enum Kind {
    Char,
    Word,
    Line,
}

#[inline]
fn run(textarea: &TextArea<'_>, kind: Kind) {
    let mut term = dummy_terminal();
    let mut t = textarea.clone();
    t.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
    for _ in 0..100 {
        let modified = match kind {
            Kind::Char => t.delete_char(),
            Kind::Word => t.delete_word(),
            Kind::Line => t.delete_line_by_head(),
        };
        if !modified {
            t = textarea.clone();
            t.move_cursor(CursorMove::Jump(u16::MAX, u16::MAX));
        }
        term.draw_textarea(&t);
    }
}

fn bench(c: &mut Criterion) {
    let mut lines = vec![];
    for _ in 0..10 {
        lines.extend(LOREM.iter().map(|s| s.to_string()));
    }
    let textarea = TextArea::new(lines);

    c.bench_function("delete::char", |b| b.iter(|| run(&textarea, Kind::Char)));
    c.bench_function("delete::word", |b| b.iter(|| run(&textarea, Kind::Word)));
    c.bench_function("delete::line", |b| b.iter(|| run(&textarea, Kind::Line)));
}

criterion_group!(delete, bench);
criterion_main!(delete);
