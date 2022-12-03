pub mod app;
pub mod component;
pub mod model;
pub mod parser;
pub mod ui;

use anyhow::Error;
use anyhow::Result;
use app::config::map_key_event;
use app::config::Config;
use app::config::KeyName;
use app::loader::FileLoader;
use chrono::Local;
use clap::Parser;
use crossterm::event;
use crossterm::event::poll;
use crossterm::event::Event;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use model::time::RealTimeFactory;
use std::io;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    path: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let path = &args.path;

    let mut stdout = io::stdout();
    execute!(stdout)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    enable_raw_mode()?;
    terminal.clear()?;
    let now = Local::now().naive_local();

    let config: Config = confy::load("pttlog", "config").expect("Could not load config");
    let mut app = app::App::new(
        FileLoader::new(path.to_string(), &config),
        &config,
        &RealTimeFactory {},
        &now,
    );
    app.reload();

    loop {
        let cmd = main_loop(&mut terminal, &mut app)?;
        match cmd {
            Cmd::Quit => break,
        };
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(),)?;

    Ok(())
}

enum Cmd {
    Quit,
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut app::App,
) -> Result<Cmd, Error> {
    loop {
        terminal.draw(|f| app.draw(f).expect("Could not draw"))?;

        if (poll(Duration::from_millis(1000)))? {
            if let Event::Key(key) = event::read()? {
                let key = map_key_event(key);
                match key.name {
                    KeyName::Quit => return Ok(Cmd::Quit),
                    _ => {
                        app.handle(key);
                    }
                }
            }
        }
    }
}
