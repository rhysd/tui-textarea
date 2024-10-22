// Use `termion` v1.5 for `tui` backend.
use termion_15 as termion;

use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Event as TermEvent;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

enum Event {
    Term(TermEvent),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let events = {
        let events = io::stdin().events();
        let (tx, rx) = mpsc::channel();
        let keys_tx = tx.clone();
        thread::spawn(move || {
            for event in events.flatten() {
                keys_tx.send(Event::Term(event)).unwrap();
            }
        });
        thread::spawn(move || loop {
            tx.send(Event::Tick).unwrap();
            thread::sleep(Duration::from_millis(100));
        });
        rx
    };

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Termion Minimal Example"),
    );

    loop {
        match events.recv()? {
            Event::Term(event) => match event.into() {
                Input { key: Key::Esc, .. } => break,
                input => {
                    textarea.input(input);
                }
            },
            Event::Tick => {}
        }
        term.draw(|f| {
            f.render_widget(&textarea, f.size());
        })?;
    }

    drop(term); // Leave terminal raw mode to print the following line
    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
