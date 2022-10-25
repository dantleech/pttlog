mod parser;
mod ui;

use clap::Parser;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
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
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.draw(|frame| {
        let size = frame.size();
        let table = ui::table(&entries);

        frame.render_widget(table, size)
    })?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
    )?;

    Ok(())
}
