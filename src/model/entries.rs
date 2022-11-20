use std::collections::HashMap;
use std::slice::Iter;

use crate::parser::{Entries, Entry, Token, TokenKind, Tokens};
use chrono::{Datelike, Local, Timelike};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

pub struct LogDays {
    current_date: NaiveDateTime,
    entries: Entries,
    index: usize,
}

impl LogDays {
    pub fn new<'a>(entries: Entries) -> LogDays {
        LogDays {
            current_date: Local::now().naive_local(),
            entries,
            index: 0,
        }
    }

    pub(crate) fn at(&self, index: usize) -> LogDay {
        LogDay::new(&self.current_date, &self.entries.entries[index])
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.entries.len()
    }

    pub(crate) fn tag_summary(&self, tag: TokenKind) -> Vec<TagMeta> {
        self.entries.entries.iter().fold(
            HashMap::new(),
            |entry_map: HashMap<String, TagMeta>, entry: &Entry| {
                let view = LogDay::new(&self.current_date, &entry);
                entry_map
            },
        );
        vec![]
    }
}

pub struct LogDay<'a> {
    logs: Vec<LogEntry<'a>>,
    date: LogDate<'a>,
}

impl LogDay<'_> {
    pub fn iter(&self) -> Iter<LogEntry> {
        self.logs().into_iter()
    }
    pub fn new<'a>(current_date: &'a NaiveDateTime, entry: &'a Entry) -> LogDay<'a> {
        process_entry(current_date, entry)
    }

    pub fn duration_total(&self) -> LogDuration {
        LogDuration {
            duration: Duration::minutes(
                self.logs()
                    .iter()
                    .fold(0, |c, l| c + l.time_range().duration().num_minutes()),
            ),
        }
    }

    pub fn logs(&self) -> &Vec<LogEntry> {
        &self.logs
    }

    pub fn date(&self) -> &LogDate {
        &self.date
    }

    pub fn tag_summary(&self, kind: TokenKind) -> Vec<TagMeta> {
        let entry_map = self.iter().fold(
            HashMap::new(),
            |entry_map: HashMap<String, TagMeta>, log: &LogEntry| {
                log.description().by_kind(kind).iter().fold(
                    entry_map,
                    |mut acc: HashMap<String, TagMeta>, tag: &&Token| {
                        let meta = acc.entry(tag.text().to_string()).or_insert(TagMeta {
                            tag: tag.text().to_string(),
                            kind: tag.kind,
                            duration: LogDuration::from_minutes(0 as i64),
                            count: 0,
                        });
                        meta.count += 1;
                        meta.duration.duration = meta
                            .duration
                            .duration
                            .checked_add(&log.time_range().duration().duration)
                            .expect("overflow occurred");
                        acc
                    },
                )
            },
        );

        let mut tag_metas: Vec<TagMeta> = vec![];
        for (_, v) in entry_map {
            tag_metas.push(v)
        }
        tag_metas.sort_by(|a, b| b.duration.duration.cmp(&a.duration.duration));
        tag_metas
    }
}

pub struct TagMeta {
    pub tag: String,
    pub kind: TokenKind,
    pub duration: LogDuration,
    pub count: usize,
}

pub struct LogEntry<'a> {
    time_range: TimeRangeView,
    desription: &'a Tokens,
}

impl LogEntry<'_> {
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
pub struct LogDate<'a> {
    now: &'a NaiveDateTime,
    date: NaiveDate,
}

impl LogDate<'_> {
    pub fn is_today(&self) -> bool {
        return self.date.year() == self.now.year()
            && self.date.month() == self.now.month()
            && self.date.day() == self.now.day();
    }

    pub(crate) fn to_verbose_string(&self) -> String {
        self.date.format("%A %e %B, %Y").to_string()
    }
}
pub struct LogDuration {
    duration: Duration,
}
impl LogDuration {
    pub fn num_minutes(&self) -> i64 {
        self.duration.num_minutes()
    }

    fn from_minutes(arg: i64) -> LogDuration {
        LogDuration {
            duration: Duration::minutes(arg),
        }
    }
}
impl ToString for LogDuration {
    fn to_string(&self) -> String {
        let hours = self.duration.num_minutes() / 60;
        let mins = self.duration.num_minutes() % 60;
        if 0 == hours {
            return format!("{}m", mins);
        }

        format!("{}h{}m", hours, mins)
    }
}

fn process_entry<'a>(current_date: &'a NaiveDateTime, entry: &'a Entry) -> LogDay<'a> {
    let mut logs: Vec<LogEntry> = vec![];

    // # resolve the end dates
    //
    // 0. reverse entries
    // 1. if has end date, map to view, done
    // 2. if previous start date, set end date to previous start date
    // 3. if today, then end date = now
    // 4. if end date not set and not today, then end date = start date
    for log in entry.logs.iter().rev() {
        if log.time.end.is_some() {
            logs.push(LogEntry {
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
            logs.push(LogEntry {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: logs.last().unwrap().time_range().start,
                    ongoing: false,
                },
                desription: &log.description,
            });
            continue;
        }
        if current_date.date() == entry.date_object() {
            logs.push(LogEntry {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: current_date.time(),
                    ongoing: true,
                },
                desription: &log.description,
            });
            continue;
        }

        logs.push(LogEntry {
            time_range: TimeRangeView {
                start: log.time.start.time(),
                end: log.time.start.time(),
                ongoing: false,
            },
            desription: &log.description,
        });
    }
    logs.reverse();

    LogDay {
        logs,
        date: LogDate {
            now: current_date,
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

    pub fn duration(&self) -> LogDuration {
        // end is after start
        if self.end >= self.start {
            return LogDuration {
                duration: self.end - self.start,
            };
        }
        // end is before start, assume rollover
        let m_to_mid = 1440 - (self.start.hour() * 60 + self.start.minute());
        let m_past_mid = self.end.hour() * 60 + self.end.minute();

        LogDuration {
            duration: Duration::minutes(m_to_mid as i64 + m_past_mid as i64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    use crate::parser::{self, Date, Entry, Log, Time, TimeRange, Token, Tokens};

    #[test]
    fn log_view_percentage_of_day() {
        let l = LogEntry {
            time_range: TimeRangeView {
                start: NaiveTime::from_hms(0, 0, 0),
                end: NaiveTime::from_hms(12, 0, 0),
                ongoing: false,
            },
            desription: &Tokens::from_prose("foo".to_string()),
        };
        assert_eq!(50.0, l.percentage_of_day(1440));
    }

    #[test]
    fn time_range_view_duration() {
        let t = TimeRangeView {
            start: NaiveTime::from_hms(10, 30, 0),
            end: NaiveTime::from_hms(12, 0, 0),
            ongoing: false,
        };
        assert_eq!(90, t.duration().num_minutes());
    }

    #[test]
    fn time_range_view_duration_overflow() {
        let t = TimeRangeView {
            start: NaiveTime::from_hms(23, 30, 0),
            end: NaiveTime::from_hms(0, 30, 0),
            ongoing: false,
        };
        assert_eq!(60, t.duration().num_minutes());
    }

    #[test]
    fn test_calculates_duration() {
        {
            let entry = Entry {
                date: Date::from_ymd(2022, 01, 01),
                logs: vec![
                    Log {
                        time: TimeRange::from_start(Time::from_hm(10, 0)),
                        description: Tokens::from_prose("foo".to_string()),
                    },
                    Log {
                        time: TimeRange::from_start(Time::from_hm(11, 0)),
                        description: Tokens::from_prose("foo".to_string()),
                    },
                    Log {
                        time: TimeRange::from_start(Time::from_hm(13, 0)),
                        description: Tokens::from_prose("foo".to_string()),
                    },
                ],
            };
            let time = NaiveDate::from_ymd(2022, 01, 01).and_hms(0, 0, 0);
            let view = LogDay::new(&time, &entry);
            assert_eq!("10:00:00-11:00:00", view.logs[0].time_range().to_string())
        }
    }

    #[test]
    fn test_view_tag_summary() {
        let entry = Entry {
            date: Date::from_ymd(2022, 01, 01),
            logs: vec![
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(10, 30)),
                    description: Tokens::new(vec![Token::tag("foobar".to_string())]),
                },
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(11, 0)),
                    description: Tokens::new(vec![
                        Token::tag("barfoo".to_string()),
                        Token::tag("foobar".to_string()),
                    ]),
                },
            ],
        };
        let time = NaiveDate::from_ymd(2022, 01, 01).and_hms(0, 0, 0);
        let view = LogDay::new(&time, &entry);
        let summary = view.tag_summary(parser::TokenKind::Tag);

        assert_eq!(2, summary.len());

        assert_eq!("barfoo".to_string(), summary[1].tag);
        assert_eq!(1, summary[1].count);
        assert_eq!(60, summary[1].duration.num_minutes());

        assert_eq!("foobar".to_string(), summary[0].tag);
        assert_eq!(2, summary[0].count);
        assert_eq!(90, summary[0].duration.num_minutes());
    }
}
