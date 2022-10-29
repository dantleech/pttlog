mod parser;
mod ui;
mod app;

use clap::Parser;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use parser::parse;
use std::fs;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    path: String,
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

    let app = app::App::new(entries);
    let _ = run_app(&mut terminal, app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
    )?;

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: app::App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            ui::layout(frame, &app)
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('p') => app.entry_previous(),
                KeyCode::Char('n') => app.entry_next(),
                _ => {}
            }
        }
    }
}
