use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tui_textarea::{CursorMove, TextArea};
use tui_textarea_bench::{dummy_terminal, TerminalExt, LOREM};

#[derive(Clone, Copy)]
enum Restore {
    TopLeft,
    BottomLeft,
    BottomRight,
    None,
}

impl Restore {
    fn cursor_move(self) -> Option<CursorMove> {
        match self {
            Self::TopLeft => Some(CursorMove::Jump(0, 0)),
            Self::BottomLeft => Some(CursorMove::Jump(u16::MAX, 0)),
            Self::BottomRight => Some(CursorMove::Jump(u16::MAX, u16::MAX)),
            Self::None => None,
        }
    }
}

fn run(moves: &[CursorMove], restore: Restore, repeat: usize) -> (usize, usize) {
    let mut lines = Vec::with_capacity(LOREM.len() * 2 + 1);
    lines.extend(LOREM.iter().map(|s| s.to_string()));
    lines.push("".to_string());
    lines.extend(LOREM.iter().map(|s| s.to_string()));

    let mut textarea = TextArea::new(lines);
    let mut term = dummy_terminal();

    let mut prev = textarea.cursor();
    for _ in 0..repeat {
        for m in moves {
            textarea.move_cursor(*m);
            term.draw_textarea(&textarea);
        }
        if let Some(m) = restore.cursor_move() {
            if textarea.cursor() == prev {
                textarea.move_cursor(m);
                prev = textarea.cursor();
            }
        }
    }

    textarea.cursor()
}

fn move_char(c: &mut Criterion) {
    c.bench_function("cursor::char::forward", |b| {
        b.iter(|| black_box(run(&[CursorMove::Forward], Restore::TopLeft, 1000)))
    });
    c.bench_function("cursor::char::back", |b| {
        b.iter(|| black_box(run(&[CursorMove::Back], Restore::BottomRight, 1000)))
    });
    c.bench_function("cursor::char::down", |b| {
        b.iter(|| black_box(run(&[CursorMove::Down], Restore::TopLeft, 1000)))
    });
    c.bench_function("cursor::char::up", |b| {
        b.iter(|| black_box(run(&[CursorMove::Up], Restore::BottomLeft, 1000)))
    });
}
fn move_word(c: &mut Criterion) {
    c.bench_function("cursor::word::forward", |b| {
        b.iter(|| black_box(run(&[CursorMove::WordForward], Restore::TopLeft, 1000)))
    });
    c.bench_function("cursor::word::back", |b| {
        b.iter(|| black_box(run(&[CursorMove::WordBack], Restore::BottomRight, 1000)))
    });
}
fn move_paragraph(c: &mut Criterion) {
    c.bench_function("cursor::paragraph::down", |b| {
        b.iter(|| black_box(run(&[CursorMove::ParagraphForward], Restore::TopLeft, 1000)))
    });
    c.bench_function("cursor::paragraph::up", |b| {
        b.iter(|| black_box(run(&[CursorMove::ParagraphBack], Restore::BottomLeft, 1000)))
    });
}
fn move_edge(c: &mut Criterion) {
    c.bench_function("cursor::edge::head_end", |b| {
        b.iter(|| {
            black_box(run(
                &[CursorMove::End, CursorMove::Head],
                Restore::None,
                500,
            ))
        })
    });
    c.bench_function("cursor::edge::top_bottom", |b| {
        b.iter(|| {
            black_box(run(
                &[CursorMove::Bottom, CursorMove::Top],
                Restore::None,
                500,
            ))
        })
    });
}

criterion_group!(cursor, move_char, move_word, move_paragraph, move_edge);
criterion_main!(cursor);
