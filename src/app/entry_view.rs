use chrono::{NaiveDate, Duration, NaiveDateTime};
use chrono::Datelike;
use crate::parser::{TimeRange, Tokens, Entry};

use super::App;

pub struct EntryView<'a> {
    app: &'a App,
    entry: &'a Entry
}
impl EntryView<'_> {
    pub fn create<'a>(app: &'a App, entry: &'a Entry) -> EntryView<'a> {
        EntryView { app, entry }
    }

    pub fn duration_total(&self) -> i64 {
        self.logs()
            .iter()
            .fold(0, |c, l| c + l.time_range().duration().num_minutes())
    }

    pub fn logs(&self) -> Vec<LogView> {
        self.entry.logs.iter().map(|log| {
            LogView{
                time_range: &log.time,
                desription: &log.description,
            }
        }).collect()
    }

    pub fn date(&self) -> EntryDateView {
        EntryDateView { now: self.app.current_date(), date: self.entry.date_object() }
    }
}

pub struct LogView<'a> {
    time_range: &'a TimeRange,
    desription: &'a Tokens,
}
impl LogView<'_> {
    pub fn percentage_of_day(&self) -> f32 {
        0.0
    }
    pub fn time_range(&self) -> &TimeRange {
        self.time_range
    }
    pub fn duration(&self) -> DurationView {
        DurationView { duration: self.time_range.duration() }
    }

    pub(crate) fn description(&self) -> &Tokens {
        self.desription
    }
}
pub struct EntryDateView<'a> {
    now: &'a NaiveDateTime,
    date: NaiveDate
}

impl EntryDateView<'_> {
    pub fn is_today(&self) -> bool {
        return self.date.year() == self.now.year()
            && self.date.month() == self.now.month()
            && self.date.day() == self.now.day();
    }

    pub(crate) fn to_verbose_string(&self) -> String {
        self.date.format("%A %e %B, %Y").to_string()
    }


}
pub struct DurationView {
    duration: Duration
}
impl ToString for DurationView {
    fn to_string(&self) -> String {
        let hours = self.duration.num_minutes() / 60;
        let mins = self.duration.num_minutes() % 60;

        return format!("{}h{}m", hours, mins);
    }
}
