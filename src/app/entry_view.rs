use super::App;
use crate::parser::{Entry, Tokens};
use chrono::{Datelike, Timelike};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

pub struct EntryView<'a> {
    logs: Vec<LogView<'a>>,
    date: EntryDateView<'a>,
}
impl EntryView<'_> {
    pub fn create<'a>(app: &'a App, entry: &'a Entry) -> EntryView<'a> {
        process_entry(app, entry)
    }

    pub fn duration_total(&self) -> DurationView {
        DurationView{
            duration: Duration::minutes(
                self.logs()
                    .iter()
                    .fold(0, |c, l| c + l.time_range().duration().num_minutes())
            )
        }
    }

    pub fn logs(&self) -> &Vec<LogView> {
        &self.logs
    }

    pub fn date(&self) -> &EntryDateView {
        &self.date
    }
}

pub struct LogView<'a> {
    time_range: TimeRangeView,
    desription: &'a Tokens,
}
impl LogView<'_> {
    pub fn percentage_of_day(&self) -> f32 {
        0.0
    }

    pub fn time_range(&self) -> &TimeRangeView {
        &self.time_range
    }

    pub(crate) fn description(&self) -> &Tokens {
        self.desription
    }
}
pub struct EntryDateView<'a> {
    now: &'a NaiveDateTime,
    date: NaiveDate,
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
    duration: Duration,
}
impl DurationView {
    fn num_minutes(&self) -> i64 {
        self.duration.num_minutes()
    }
}
impl ToString for DurationView {
    fn to_string(&self) -> String {
        let hours = self.duration.num_minutes() / 60;
        let mins = self.duration.num_minutes() % 60;

        return format!("{}h{}m", hours, mins);
    }
}

fn process_entry<'a>(app: &'a App, entry: &'a Entry) -> EntryView {
    let mut logs: Vec<LogView> = vec![];

    // # resolve the end dates
    //
    // 0. reverse entries
    // 1. if has end date, map to view, done
    // 2. if previous start date, set end date to previous start date
    // 3. if today, then end date = now
    // 4. if end date not set and not today, then end date = start date
    for log in entry.logs.iter().rev() {
        if log.time.end.is_some() {
            logs.push(LogView {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: log.time.end.unwrap().time(),
                    ongoing: false,
                },
                desription: &log.description,
            });
            continue;
        }
        if logs.last().is_some() {
            logs.push(LogView {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: logs.last().unwrap().time_range().start,
                    ongoing: false,
                },
                desription: &log.description,
            });
            continue;
        }
        if app.current_date().date() == entry.date_object() {
            logs.push(LogView {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: app.current_date().time(),
                    ongoing: true,
                },
                desription: &log.description,
            });
            continue;
        }

        logs.push(LogView {
            time_range: TimeRangeView {
                start: log.time.start.time(),
                end: log.time.start.time(),
                ongoing: false,
            },
            desription: &log.description,
        });
    }
    logs.reverse();

    EntryView {
        logs,
        date: EntryDateView {
            now: app.current_date(),
            date: entry.date_object(),
        },
    }
}

pub struct TimeRangeView {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub ongoing: bool,
}

impl TimeRangeView {
    pub fn to_string(&self) -> String {
        format!("{}-{}", self.start, self.end)
    }
    /// Return duration elapsed between the time ranges
    /// if the end time is before the last time it is assumed that
    /// the time rolled over.
    ///
    /// ```
    /// use pttlogger::parser::{TimeRange,Time};
    ///
    /// let t = TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(11,30));
    /// assert_eq!(90, t.duration().num_minutes());
    /// ```
    ///
    /// ```
    /// use pttlogger::parser::{TimeRange,Time};
    ///
    /// let t = TimeRange::from_start_end(Time::from_hm(23, 30), Time::from_hm(0,30));
    /// assert_eq!(60, t.duration().num_minutes());
    /// ```
    pub fn duration(&self) -> DurationView {
        // end is after start
        if self.end >= self.start {
            return DurationView{duration:self.end - self.start};
        }
        // end is before start, assume rollover
        let m_to_mid = 1440 - (self.start.hour() * 60 + self.start.minute());
        let m_past_mid = self.end.hour() * 60 + self.end.minute();

        DurationView{
            duration: Duration::minutes(m_to_mid as i64 + m_past_mid as i64)
        }
    }
}
