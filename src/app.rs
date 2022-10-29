
use super::parser;

pub struct App {
    current_entry: usize,
    pub entries: parser::Entries,
}

impl App {
    pub fn new(entries: parser::Entries) -> App {
        App{
            current_entry: entries.entries.len() - 1,
            entries,
        }
    }

    pub fn current_entry(&self) -> &parser::Entry {
        &self.entries.entries[self.current_entry]
    }
}

