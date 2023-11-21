use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;

use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use simplelog::*;
use std::fs::File;
use std::io;
use tui_textarea::{trace, Input, Key, TextArea, WrapMode};
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
    //textarea.set_line_number_style(Style::default());
    let title = String::from("Crossterm Minimal Example");

    textarea.set_block(Block::default().borders(Borders::ALL).title(title.as_str()));
    let mut mouse_pos = String::new();
    let mut mouse_event = String::new();
    // let mut draw = true;
    loop {
        term.draw(|f| {
            let t = format!(
                "Screen {:?} Data: {:?} VP: {:?}\n{}\n{}",
                textarea.pub_screen_cursor(),
                textarea.cursor(),
                textarea.viewport(),
                mouse_pos,
                mouse_event
            );
            let r = Rect::new(0, 0, f.size().width, 5);

            let para = Paragraph::new(t).block(Block::default().title(""));
            f.render_widget(para, r);

            let mut rect = f.size();
            rect.y += 5;
            rect.height -= 5;
            f.render_widget(textarea.widget(), rect);
        })?;
        match crossterm::event::read()? {
            crossterm::event::Event::Mouse(me) => {
                trace!("Mouse: {:?}", me);
                mouse_event = format!("MouseEvent: {:?}", me);
                match me.kind {
                    crossterm::event::MouseEventKind::Moved => {
                        let dc =
                            textarea.abs_screen_to_data_cursor(me.row as usize, me.column as usize);
                        if let Some(c) = dc {
                            let char = textarea.lines()[c.0].chars().nth(c.1);
                            mouse_pos = format!("Mouse: {:?} {:?}", dc, char);
                        } else {
                            mouse_pos = String::new();
                        }
                    }
                    crossterm::event::MouseEventKind::Down(_button) => {
                        // ignore
                    }
                    _ => {}
                }
            }
            crossterm::event::Event::Key(ke) if ke.kind == KeyEventKind::Release => {
                //    ignore
                //draw = false;
            }
            ev => match ev.into() {
                Input { key: Key::Esc, .. } => break,

                Input {
                    key: Key::Char('R'),
                    ctrl: true,
                    shift: true,
                    alt: false,
                } => textarea.set_wrap_mode(WrapMode::Word),
                Input {
                    key: Key::Char('Z'),
                    ctrl: true,
                    shift: true,
                    alt: false,
                } => textarea.set_wrap_mode(WrapMode::None),
                Input {
                    key: Key::Char('F'),
                    ctrl: true,
                    shift: true,
                    alt: false,
                } => textarea.set_wrap_mode(WrapMode::Char),

                input => {
                    if input.key != Key::Null {
                        textarea.input(input.clone());
                    };
                }
            },
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
