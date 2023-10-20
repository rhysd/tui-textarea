use ratatui::backend::TermwizBackend;
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use std::error::Error;
use std::time::{Duration, Instant};
use termwiz::input::InputEvent;
use termwiz::terminal::Terminal as TermwizTerminal;
use tui_textarea::{Input, Key, TextArea};

const TICK_RATE: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn Error>> {
    let backend = TermwizBackend::new()?;
    let mut term = Terminal::new(backend)?;
    term.hide_cursor()?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Termwiz Minimal Example"),
    );
    let mut last_tick = Instant::now();

    // The event loop
    loop {
        term.draw(|f| {
            let widget = textarea.widget();
            f.render_widget(widget, f.size());
        })?;

        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if let Some(input) = term
            .backend_mut()
            .buffered_terminal_mut()
            .terminal()
            .poll_input(Some(timeout))?
        {
            if let InputEvent::Resized { cols, rows } = input {
                term.backend_mut()
                    .buffered_terminal_mut()
                    .resize(cols, rows);
            } else {
                match input.into() {
                    Input { key: Key::Esc, .. } => break,
                    input => {
                        textarea.input(input);
                    }
                }
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            last_tick = Instant::now();
        }
    }

    term.show_cursor()?;
    term.flush()?;
    drop(term); // Leave terminal raw mode to print the following line

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
