use crate::parser::Entries;
use crate::parser::Entry;
use crate::parser::parse;
use std::fs;

pub trait Loader {
    fn load(&self) -> Entries;
}

pub struct FileLoader {
    pub path: String,
}

impl Loader for FileLoader {
    fn load(&self) -> Entries {
        let contents = fs::read_to_string(&self.path).expect("Could not read file");
        let (_, entries) = parse(&contents).expect("Could not parse file");
        if entries.entries.len() == 0 {
            return Entries{entries: vec![Entry::placeholder()]};
        }
        entries
    }
}

pub struct VecLoader {
    pub entries: Box<dyn Fn() -> Entries>

}

impl Loader for VecLoader {
    fn load(&self) -> Entries {
        return (self.entries)()
    }
}
