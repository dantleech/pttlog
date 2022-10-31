mod app;
mod parser;
mod ui;

use clap::Parser;
use crossterm::event;
use crossterm::event::poll;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use parser::parse;
use std::fs;
use std::io;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    path: String,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let path = &args.path;

    let mut stdout = io::stdout();
    execute!(stdout)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    terminal.clear()?;
    let mut app = app::App::new(load_entries(path)?);

    loop {
        let cmd = main_loop(&mut terminal, &mut app)?;
        match cmd {
            Cmd::Quit => break,
            Cmd::Reload => {
                app.with_entries(load_entries(path)?);
                continue;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(),)?;

    Ok(())
}

fn load_entries(path: &str) -> io::Result<parser::Entries> {
    let contents = fs::read_to_string(path)?;
    let (_, entries) = parse(&contents).expect("Could not parse file");
    Ok(entries)
}

enum Cmd {
    Quit,
    Reload,
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::App,
) -> io::Result<Cmd> {
    loop {
        app.tick();
        terminal.draw(|frame| ui::layout(frame, app))?;
        if (poll(Duration::from_millis(10)))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(Cmd::Quit),
                    KeyCode::Char('p') => app.entry_previous(),
                    KeyCode::Char('n') => app.entry_next(),
                    KeyCode::Char('r') => {
                        app.notify("Reloaded timesheet".to_string(), 50);
                        return Ok(Cmd::Reload);
                    }
                    _ => {}
                }
            }
        }
    }
}
