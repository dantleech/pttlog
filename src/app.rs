use crate::{component::log_table::LogTable, model::entries::LogDays};

use super::component::day::Day;

use self::{config::Config, entry_view::EntryView};
use super::parser;
use chrono::{Local, NaiveDateTime};
pub mod config;
pub mod entry_view;
pub mod loader;

pub struct App<'a> {
    pub iteration: u8,
    current_time: NaiveDateTime,
    current_entry: usize,
    pub log_days: LogDays<'a>,
    pub notification: Notification,
    loader: Box<dyn loader::Loader + 'a>,
    _config: &'a Config,
    pub day: Day<'a>,
}

impl App<'_> {
    pub fn new<'a>(loader: Box<dyn loader::Loader + 'a>, config: &'a Config) -> App<'a> {
        let log_days = LogDays::new(
            Local::now().naive_local(),
            &parser::Entries {
                entries: vec![parser::Entry::placeholder()],
            },
        );
        App {
            iteration: 0,
            current_time: Local::now().naive_local(),
            loader,
            _config: config,
            current_entry: 0,
            log_days,
            notification: Notification {
                notification: "".to_string(),
                lifetime: 0,
            },
            day: Day::new(log_days) {
                entries: log_days,
                index: 0,
                log_table: LogTable::new(log_days },
                tag_summary: TagSummaryTable {},
                ticket_summary: (),
            },
        }
    }

    pub fn current_entry_number(&self) -> usize {
        return self.current_entry + 1;
    }

    pub fn current_entry(&self) -> EntryView {
        EntryView::create(&self, &self.log_days.entries[self.current_entry])
    }
    pub fn entry_count(&self) -> usize {
        self.log_days.entries.len()
    }

    pub fn entry_previous(&mut self) {
        if self.current_entry == 0 {
            return;
        }
        self.current_entry -= 1;
    }

    pub fn entry_next(&mut self) {
        if self.current_entry == self.log_days.entries.len() - 1 {
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
        self.log_days = entries;
    }

    pub fn reload(&mut self) {
        self.log_days = self.loader.load();
        self.current_entry = self.log_days.entries.len() - 1
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

    use super::{config::Config, loader::FuncLoader, App};

    #[test]
    pub fn test_replace_entries_resets_current_entry_if_out_of_bounds() {
        let config = Config::empty();
        let mut app = App::new(
            FuncLoader::new(Box::new(|| parser::Entries {
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
            })),
            &config,
        );
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
