use super::parser;

pub struct App {
    current_entry: usize,
    pub entries: parser::Entries,
    pub notification: Notification,
}

impl App {
    pub fn new(entries: parser::Entries) -> App {
        App {
            current_entry: entries.entries.len() - 1,
            entries,
            notification: Notification {
                notification: "".to_string(),
                lifetime: 0,
            },
        }
    }

    pub fn current_entry_number(&self) -> usize {
        return self.current_entry + 1;
    }

    pub fn current_entry(&mut self) -> &parser::Entry {
        if self.current_entry >= self.entries.entries.len() {
            self.current_entry = self.entries.entries.len() - 1;
        }
        &self.entries.entries[self.current_entry]
    }
    pub fn entry_count(&self) -> usize {
        self.entries.entries.len()
    }

    pub(crate) fn entry_previous(&mut self) {
        if self.current_entry == 0 {
            return;
        }
        self.current_entry -= 1;
    }

    pub(crate) fn entry_next(&mut self) {
        if self.current_entry == self.entries.entries.len() - 1 {
            return;
        }
        self.current_entry += 1;
    }

    pub(crate) fn notify(&mut self, message: String, lifetime: i16) {
        self.notification.notification = message;
        self.notification.lifetime = lifetime;
    }

    pub fn tick(&mut self) {
        self.notification.tick();
    }

    pub(crate) fn with_entries(&mut self, entries: parser::Entries) {
        self.entries = entries;
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

    use super::App;

    #[test]
    pub fn test_replace_entries_resets_current_entry_if_out_of_bounds()
    {
        let mut app = App::new(parser::Entries{ entries: vec![
            Entry{
                date: parser::Date { year: 2022, month: 01, day: 01 },
                logs: vec![]
            },
            Entry{
                date: parser::Date { year: 2022, month: 01, day: 02 },
                logs: vec![]
            },
        ]});
        app.entry_next();
        app.with_entries(parser::Entries{ entries: vec![
            Entry{
                date: parser::Date { year: 2022, month: 01, day: 01 },
                logs: vec![]
            },
        ]});
        app.current_entry();
    }
}
