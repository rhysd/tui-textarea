use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_password_mode();
    let layout =
        Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)].as_slice());
    textarea.set_style(Style::default().fg(Color::LightGreen));
    textarea.set_block(Block::default().borders(Borders::ALL).title("Passord"));
    loop {
        term.draw(|f| {
            let chunks = layout.split(f.size());
            let widget = textarea.widget();
            f.render_widget(widget, chunks[0]);
        })?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            Input {
                key: Key::Enter, ..
            } => break,

            input => {
                // TextArea::input returns if the input modified its text
                if textarea.input(input) {
                    //  is_valid = validate(&mut textarea);
                }
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

    println!("Input: {:?}", textarea.lines()[0]);
    Ok(())
}
