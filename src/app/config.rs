use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_derive::{Deserialize, Serialize};
use iso_currency::{Currency};


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub projects: Vec<Project>,
}

impl Config {
    pub fn empty() -> Config {
        Config { projects: vec![] }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub ticket_prefix: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rate: Option<Rate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rate {
    pub rate: u64,
    pub currency: Currency,
}

pub enum KeyName {
    NextTab,
    PreviousPage,
    NextPage,
    Unknown,
    Quit,
    Reload,
    DayView,
    WeekView,
    MonthView,
    YearView,
    ToggleFilter,
    PrevTab,
}

pub struct Key {
    pub name: KeyName,
    pub event: KeyEvent,
}

impl Key {
    pub fn for_key_code(code: KeyCode) -> Self {
        let key = KeyEvent::new(code, KeyModifiers::empty());
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
            KeyCode::Tab => KeyName::NextTab,
            KeyCode::BackTab => KeyName::PrevTab,
            KeyCode::Char('w') => KeyName::WeekView,
            KeyCode::Char('m') => KeyName::MonthView,
            KeyCode::Char('d') => KeyName::DayView,
            KeyCode::Char('y') => KeyName::YearView,
            KeyCode::Char('f') => KeyName::ToggleFilter,
            _ => KeyName::Unknown,
        },
        event: key,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn test_example_config() {
        let config: Config = confy::load_path(Path::new("example/example_config.toml")).unwrap();

        assert_eq!(config.projects[0].name, "Project One".to_string());
        assert_eq!(config.projects[1].name, "Project Two".to_string());
        assert_eq!(10000, config.projects[1].rate.clone().unwrap().rate);
        assert_eq!(config.projects[1].rate.clone().unwrap().currency, Currency::GBP);
    }
}
