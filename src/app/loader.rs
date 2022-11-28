use anyhow::bail;
use anyhow::Result;

use crate::parser::parse;
use crate::parser::Entries;
use crate::parser::Entry;
use std::fs;

use super::config::Config;

pub trait Loader {
    fn load(&self) -> Result<Entries, anyhow::Error>;
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
    fn load(&self) -> Result<Entries, anyhow::Error> {
        let contents = fs::read_to_string(&self.path)?;
        let entries = match parse(&contents, &self.config) {
            Ok((_, ok)) => ok,
            Err(err) => bail!(err.to_string()),
        };

        if entries.entries.len() == 0 {
            return Ok(Entries {
                entries: vec![Entry::placeholder()],
            });
        }
        Ok(entries)
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
    fn load(&self) -> Result<Entries, anyhow::Error> {
        Ok((self.factory)())
    }
}
