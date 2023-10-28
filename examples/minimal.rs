use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use simplelog::*;
use std::fs::File;
use std::io;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};
fn main() -> io::Result<()> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        File::create("my_rust_binary.log").unwrap(),
    )])
    .unwrap();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Crossterm Minimal Example"),
    );

    loop {
        term.draw(|f| {
            f.render_widget(textarea.widget(), f.size());
        })?;
        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            input => {
                textarea.input(input);
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
