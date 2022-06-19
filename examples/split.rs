use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::io;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui_textarea::{Input, Key, TextArea};

fn inactivate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(Style::default());
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(
        b.style(Style::default().fg(Color::DarkGray))
            .title(" Inactive (^X to switch) "),
    );
}

fn activate(textarea: &mut TextArea<'_>) {
    textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    let b = textarea
        .block()
        .cloned()
        .unwrap_or_else(|| Block::default().borders(Borders::ALL));
    textarea.set_block(b.style(Style::default()).title(" Active "));
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = [TextArea::default(), TextArea::default()];

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

    let mut which = 0;
    activate(&mut textarea[0]);
    inactivate(&mut textarea[1]);

    loop {
        term.draw(|f| {
            let chunks = layout.split(f.size());
            for (textarea, chunk) in textarea.iter().zip(chunks) {
                let widget = textarea.widget();
                f.render_widget(widget, chunk);
            }
        })?;
        match Input::from(crossterm::event::read()?) {
            Input { key: Key::Esc, .. } => break,
            Input {
                key: Key::Char('x'),
                ctrl: true,
                ..
            } => {
                inactivate(&mut textarea[which]);
                which = (which + 1) % 2;
                activate(&mut textarea[which]);
            }
            input => {
                textarea[which].input(input);
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

    println!("Left textarea: {:?}", textarea[0].lines());
    println!("Right textarea: {:?}", textarea[1].lines());
    Ok(())
}
