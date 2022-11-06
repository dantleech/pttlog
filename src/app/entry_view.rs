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
    pub fn percentage_of_day(&self, day_total: i64) -> f64 {
        return (self.time_range.duration().num_minutes() as f64 / day_total as f64) * 100.0;
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
    pub fn num_minutes(&self) -> i64 {
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
    ///
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

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use crate::{app::{entry_view::{EntryView, LogView, TimeRangeView}, App, loader::FuncLoader}, parser::{self, Entry, Log, TimeRange, Tokens, Time, Date}};

    #[test]
    fn log_view_percentage_of_day() {
        let l = LogView{
            time_range: TimeRangeView{
                start: NaiveTime::from_hms(0,0,0),
                end: NaiveTime::from_hms(12,0,0),
                ongoing: false,
            },
            desription: &Tokens::from_prose("foo".to_string())
        };
        assert_eq!(50.0, l.percentage_of_day(1440));
    }

    #[test]
    fn time_range_view_duration() {
        let t = TimeRangeView{
            start: NaiveTime::from_hms(10,30,0),
            end: NaiveTime::from_hms(12,0,0),
            ongoing: false,
        };
        assert_eq!(90, t.duration().num_minutes());
    }

    #[test]
    fn time_range_view_duration_overflow() {
        let t = TimeRangeView{
            start: NaiveTime::from_hms(23,30, 0),
            end: NaiveTime::from_hms(0, 30, 0),
            ongoing: false,
        };
        assert_eq!(60, t.duration().num_minutes());
    }

    #[test]
    fn test_calculates_duration() {
        {
            let app = App::new(FuncLoader::new(Box::new(|| parser::Entries {
                entries: vec![
                ],
            })));
            let entry = Entry {
                date: Date::from_ymd(2022, 01, 01),
                logs: vec![
                    Log{
                        time: TimeRange::from_start(Time::from_hm(10, 0)),
                        description: Tokens::from_prose("foo".to_string())
                    },
                    Log{
                        time: TimeRange::from_start(Time::from_hm(11, 0)),
                        description: Tokens::from_prose("foo".to_string())
                    },
                    Log{
                        time: TimeRange::from_start(Time::from_hm(13, 0)),
                        description: Tokens::from_prose("foo".to_string())
                    },
                ],
            };
            let view = EntryView::create(&app, &entry);
            assert_eq!("10:00:00-11:00:00", view.logs[0].time_range().to_string())
        }
    }
}
