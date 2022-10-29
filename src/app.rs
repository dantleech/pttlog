
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

    pub fn current_entry_index(&self) -> usize {
        return self.current_entry;
    }

    pub fn current_entry(&self) -> &parser::Entry {
        &self.entries.entries[self.current_entry]
    }

    pub(crate) fn entry_previous(&mut self) {
        if self.current_entry == 0 {
            return;
        }
        self.current_entry -=1;
    }

    pub(crate) fn entry_next(&mut self) {
        if self.current_entry == self.entries.entries.len() - 1 {
            println!("{:?}", self.entries.entries);
            return;
        }
        self.current_entry +=1;
    }
}

