use chrono::{Local, NaiveDateTime};

use super::parser;

pub mod loader;

pub struct App {
    pub iteration: u8,
    current_time: NaiveDateTime,
    current_entry: usize,
    pub entries: parser::Entries,
    pub notification: Notification,
    loader: Box<dyn loader::Loader>,
}

impl App {
    pub fn new(loader: Box<dyn loader::Loader>) -> App {
        App {
            iteration: 0,
            current_time: Local::now().naive_local(),
            loader,
            current_entry: 0,
            entries: parser::Entries {
                entries: vec![parser::Entry::placeholder()],
            },
            notification: Notification {
                notification: "".to_string(),
                lifetime: 0,
            },
        }
    }

    pub fn current_entry_number(&self) -> usize {
        return self.current_entry + 1;
    }

    pub fn current_entry(&self) -> &parser::Entry {
        &self.entries.entries[self.current_entry]
    }
    pub fn entry_count(&self) -> usize {
        self.entries.entries.len()
    }

    pub fn entry_previous(&mut self) {
        if self.current_entry == 0 {
            return;
        }
        self.current_entry -= 1;
    }

    pub fn entry_next(&mut self) {
        if self.current_entry == self.entries.entries.len() - 1 {
            return;
        }
        self.current_entry += 1;
    }

    pub fn notify(&mut self, message: String, lifetime: i16) {
        self.notification.notification = message;
        self.notification.lifetime = lifetime;
    }

    pub fn tick(&mut self) {
        self.current_time = Local::now().naive_local();
        self.notification.tick();
        self.iteration = (self.iteration % 127) + 1;
    }

    pub fn with_entries(&mut self, entries: parser::Entries) {
        self.entries = entries;
    }

    pub fn reload(&mut self) {
        self.entries = self.loader.load();
        self.current_entry = self.entries.entries.len() - 1
    }

    pub fn current_date(&self) -> &NaiveDateTime {
        &self.current_time
    }
}

#[derive(Debug)]
pub struct Notification {
    pub notification: String,
    lifetime: i16,
}

impl Notification {
    fn tick(&mut self) {
        if self.lifetime > 0 {
            self.lifetime -= 1
        }
    }
    pub fn should_display(&self) -> bool {
        return self.lifetime > 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{self, Entry};

    use super::{loader::FuncLoader, App};

    #[test]
    pub fn test_replace_entries_resets_current_entry_if_out_of_bounds() {
        let mut app = App::new(FuncLoader::new(Box::new(|| parser::Entries {
            entries: vec![
                Entry {
                    date: parser::Date::from_ymd(2022, 01, 01),
                    logs: vec![],
                },
                Entry {
                    date: parser::Date::from_ymd(2022, 01, 02),
                    logs: vec![],
                },
            ],
        })));
        app.entry_next();
        app.with_entries(parser::Entries {
            entries: vec![Entry {
                date: parser::Date::from_ymd(2022, 01, 01),
                logs: vec![],
            }],
        });
        app.current_entry();
    }
}
