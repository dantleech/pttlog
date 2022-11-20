use crossterm::event::{KeyCode, KeyEvent};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub projects: Vec<Project>,
}

impl Config {
    pub fn empty() -> Config {
        Config { projects: vec![] }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub ticket_prefix: String,
    pub tags: Vec<String>,
}

pub enum KeyMap {
    PreviousPage,
    NextPage,
    Unknown,
    Quit,
    Reload,
}

pub fn map_key_event(key: KeyEvent) -> KeyMap {
    match key.code {
        KeyCode::Char('q') => KeyMap::Quit,
        KeyCode::Char('r') => KeyMap::Reload,
        KeyCode::Char('n') => KeyMap::NextPage,
        KeyCode::Char('p') => KeyMap::PreviousPage,
        _ => KeyMap::Unknown,
    }
}
