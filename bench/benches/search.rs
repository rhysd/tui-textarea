use criterion::{criterion_group, criterion_main, Criterion};
use tui_textarea::TextArea;
use tui_textarea_bench::{dummy_terminal, TerminalExt, LOREM};

fn forward(c: &mut Criterion) {
    #[inline]
    fn run(pat: &str, lines: Vec<String>) {
        let mut term = dummy_terminal();
        let mut textarea: TextArea = TextArea::new(lines);
        textarea.set_search_pattern(pat).unwrap();
        term.draw_textarea(&textarea);
        for _ in 0..100 {
            textarea.search_forward(false);
            term.draw_textarea(&textarea);
        }
        textarea.set_search_pattern(r"").unwrap();
        term.draw_textarea(&textarea);
    }

    c.bench_function("search::forward_short", |b| {
        let lines: Vec<_> = LOREM.iter().map(|s| s.to_string()).collect();
        b.iter(move || run(r"\w*i\w*", lines.clone()))
    });
    c.bench_function("search::forward_long", |b| {
        let mut lines = vec![];
        for _ in 0..10 {
            lines.extend(LOREM.iter().map(|s| s.to_string()));
        }
        b.iter(move || run(r"[A-Z]\w*", lines.clone()))
    });
}

fn back(c: &mut Criterion) {
    #[inline]
    fn run(pat: &str, lines: Vec<String>) {
        let mut term = dummy_terminal();
        let mut textarea: TextArea = TextArea::new(lines);
        textarea.set_search_pattern(pat).unwrap();
        term.draw_textarea(&textarea);
        for _ in 0..100 {
            textarea.search_back(false);
            term.draw_textarea(&textarea);
        }
        textarea.set_search_pattern(r"").unwrap();
        term.draw_textarea(&textarea);
    }

    c.bench_function("search::back_short", |b| {
        let lines: Vec<_> = LOREM.iter().map(|s| s.to_string()).collect();
        b.iter(move || run(r"\w*i\w*", lines.clone()))
    });
    c.bench_function("search::back_long", |b| {
        let mut lines = vec![];
        for _ in 0..10 {
            lines.extend(LOREM.iter().map(|s| s.to_string()));
        }
        b.iter(move || run(r"[A-Z]\w*", lines.clone()))
    });
}

criterion_group!(search, forward, back);
criterion_main!(search);
