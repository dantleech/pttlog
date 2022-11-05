use crate::parser::Entries;
use crate::parser::Entry;
use crate::parser::parse;
use std::fs;

pub trait Loader {
    fn load(&self) -> Entries;
}

pub struct FileLoader {
    path: String,
}
impl FileLoader {
    pub fn new(path: String) -> Box<dyn Loader> {
        Box::new(FileLoader{path})
    }
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

pub struct FuncLoader {
    pub factory: Box<dyn Fn() -> Entries>

}
impl FuncLoader {
    pub fn new(func: Box<dyn Fn() -> Entries>) -> Box<dyn Loader> {
        Box::new(FuncLoader{factory: func})
    }
}

impl Loader for FuncLoader {
    fn load(&self) -> Entries {
        return (self.factory)()
    }
}
