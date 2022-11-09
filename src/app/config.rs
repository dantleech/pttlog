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
