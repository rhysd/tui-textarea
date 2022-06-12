use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui_textarea::TextArea;

enum Event {
    Key(Key),
    Tick,
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let events = {
        let keys = io::stdin().keys();
        let (tx, rx) = mpsc::channel();
        let keys_tx = tx.clone();
        thread::spawn(move || {
            for key in keys.flatten() {
                keys_tx.send(Event::Key(key)).unwrap();
            }
        });
        thread::spawn(move || loop {
            tx.send(Event::Tick).unwrap();
            thread::sleep(Duration::from_millis(100));
        });
        rx
    };

    let mut textarea = TextArea::default();
    textarea.set_block(Block::default().borders(Borders::ALL).title("EXAMPLE"));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)].as_ref());

    loop {
        match events.recv()? {
            Event::Key(Key::Esc) => break,
            Event::Key(key) => textarea.input(key),
            Event::Tick => {}
        }
        term.draw(|f| {
            let chunks = layout.split(f.size());
            let widget = textarea.widget();
            f.render_widget(widget, chunks[0]);
        })?;
    }

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
