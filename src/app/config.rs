use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

pub enum KeyName {
    PreviousPage,
    NextPage,
    Unknown,
    Quit,
    Reload,
    DayView,
    WeekView,
    YearView,
}

pub struct Key {
    pub name: KeyName,
    pub event: KeyEvent,
}

impl Key {
    fn for_key_code(code: KeyCode) -> Self {
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        map_key_event(key)
    }
}

pub fn map_key_event(key: KeyEvent) -> Key {
    Key {
        name: match key.code {
            KeyCode::Char('q') => KeyName::Quit,
            KeyCode::Char('r') => KeyName::Reload,
            KeyCode::Char('n') => KeyName::NextPage,
            KeyCode::Char('p') => KeyName::PreviousPage,
            KeyCode::Char('w') => KeyName::WeekView,
            KeyCode::Char('d') => KeyName::DayView,
            KeyCode::Char('y') => KeyName::YearView,
            _ => KeyName::Unknown,
        },
        event: key,
    }
}
