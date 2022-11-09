use crate::parser::parse;
use crate::parser::Entries;
use crate::parser::Entry;
use std::fs;

use super::config::Config;

pub trait Loader {
    fn load(&self) -> Entries;
}

pub struct FileLoader<'a> {
    path: String,
    config: &'a Config,
}
impl FileLoader<'_> {
    pub fn new<'a>(path: String, config: &'a Config) -> Box<dyn Loader + 'a> {
        Box::new(FileLoader { path, config })
    }
}

impl Loader for FileLoader<'_> {
    fn load(&self) -> Entries {
        let contents = fs::read_to_string(&self.path).expect("Could not read file");
        let (_, entries) = parse(&contents, &self.config).expect("Could not parse file");
        if entries.entries.len() == 0 {
            return Entries {
                entries: vec![Entry::placeholder()],
            };
        }
        entries
    }
}

pub struct FuncLoader {
    pub factory: Box<dyn Fn() -> Entries>,
}
impl FuncLoader {
    pub fn new(func: Box<dyn Fn() -> Entries>) -> Box<dyn Loader> {
        Box::new(FuncLoader { factory: func })
    }
}

impl Loader for FuncLoader {
    fn load(&self) -> Entries {
        return (self.factory)();
    }
}
