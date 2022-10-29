mod parser;
mod ui;

use clap::Parser;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use parser::parse;
use tui::layout::Rect;
use std::fs;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    path: String,
}

struct App {
    entries: parser::Entries,
}

impl App {
    fn new(entries: parser::Entries) -> App {
        App{
            entries,
        }
    }
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let path = &args.path;
    let contents = fs::read_to_string(path)?;
    let (_, entries) = parse(&contents).expect("Could not parse file");

    let mut stdout = io::stdout();
    execute!(stdout)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = App::new(entries);
    let _ = run_app(&mut terminal, app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
    )?;

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            ui::layout(frame, &app.entries)
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
