use std::collections::HashMap;
use std::fmt::Display;
use std::slice::Iter;
use num_format::{Locale, ToFormattedString};

use crate::model::rates::Rate;
use crate::model::rates::Rates;
use crate::parser::filter::Filter;
use crate::parser::timesheet::{Entry, Tokens};
use crate::parser::token::{Token, TokenKind};
use chrono::{Datelike, Local, Timelike};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use iso_currency::Currency;
use itertools::Itertools;

#[derive(Default)]
pub struct LogContext {
    pub log_days: LogDays,
    pub rates: Rates
}

impl LogContext {
    pub fn new(log_days: LogDays, rates: Rates) -> Self {
        Self{log_days, rates}
    }

    pub(crate) fn with_log_days(&self, log_days: LogDays) -> LogContext {
        LogContext { log_days, rates: self.rates.clone() }
    }
}

#[derive(Clone, Default)]
pub struct LogDays {
    log_days: Vec<LogDay>,
}

impl LogDays {
    pub fn new<'a>(entries: Vec<Entry>) -> LogDays {
        LogDays {
            log_days: entries
                .into_iter()
                .map(|entry| LogDay::from_entry(
                    Local::now().naive_local(),
                    entry.clone(),
                ))
                .collect(),
        }
    }

    pub fn duration_total(&self) -> LogDuration {
        LogDuration {
            duration: Duration::minutes(
                self.log_days
                    .iter()
                    .fold(0, |c, e| c + e.duration_total().num_minutes()),
            ),
        }
    }

    pub fn iter(&self) -> Iter<'_, LogDay> {
        self.log_days.iter()
    }

    pub fn filter(&self, filter: &Filter) -> Self {
        LogDays {
            log_days: self
                .log_days
                .iter()
                .map(|entry| entry.with_filter(filter))
                .collect(),
        }
    }

    pub(crate) fn at(&self, index: usize) -> &LogDay {
        &self.log_days[index]
    }

    pub(crate) fn len(&self) -> usize {
        self.log_days.len()
    }

    pub(crate) fn tag_summary(&self, tag: TokenKind, context: &LogContext) -> TagSummaries {
        TagSummaries::from_log_days(&self.log_days, context, tag)
    }

    pub(crate) fn until(&self, date_start: NaiveDate, date_end: NaiveDate) -> LogDays {
        LogDays {
            log_days: self
                .log_days
                .iter()
                .filter(|entry| {
                    let date = entry.date().date;
                    date >= date_start && date < date_end
                })
                .cloned()
                .collect(),
        }
    }

    pub(crate) fn minutes_by_weekday(&self) -> Vec<(&str, u64)> {
        let counts = self.log_days.iter().fold(
            HashMap::from([
                ("Mon", 0),
                ("Tue", 0),
                ("Wed", 0),
                ("Thu", 0),
                ("Fri", 0),
                ("Sat", 0),
                ("Sun", 0),
            ]),
            |mut counts: HashMap<&str, u64>, view: &LogDay| {
                let key = view.date().date.weekday().to_string();
                let count = counts.get_mut(key.as_str()).expect("Out of bounds");
                *count += view.duration_total().duration.num_minutes().unsigned_abs();
                counts
            },
        );
        let tuples = vec![
            ("Mon", *counts.get("Mon").unwrap()),
            ("Tue", *counts.get("Tue").unwrap()),
            ("Wed", *counts.get("Wed").unwrap()),
            ("Thu", *counts.get("Thu").unwrap()),
            ("Fri", *counts.get("Fri").unwrap()),
            ("Sat", *counts.get("Sat").unwrap()),
            ("Sun", *counts.get("Sun").unwrap()),
        ];

        tuples
    }
}

#[derive(Clone)]
pub struct LogDay {
    logs: Vec<LogEntry>,
    date: LogDate,
}

impl LogDay {
    pub fn iter(&self) -> Iter<'_, LogEntry> {
        self.logs().iter()
    }

    pub fn from_entry<'a>(current_date: NaiveDateTime, entry: Entry) -> Self {
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
                    desription: log.description.clone(),
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
                    desription: log.description.clone(),
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
                    desription: log.description.clone(),
                });
                continue;
            }

            logs.push(LogEntry {
                time_range: TimeRangeView {
                    start: log.time.start.time(),
                    end: log.time.start.time(),
                    ongoing: false,
                },
                desription: log.description.clone(),
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

    pub fn new<'a>(current_date: NaiveDateTime, entry: Entry) -> LogDay {
        LogDay::from_entry(current_date, entry)
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

    pub fn tag_summary(&self, kind: TokenKind, context: &LogContext) -> TagSummaries {
        let summary_map = self.iter().fold(
            HashMap::new(),
            |entry_map: HashMap<String, TagSummary>, log: &LogEntry| {
                log.description().by_kind_refs(kind).iter().fold(
                    entry_map,
                    |mut acc: HashMap<String, TagSummary>, tag: &&Token| {
                        let meta = acc.entry(tag.text().to_string()).or_insert(TagSummary::from_tag_name_and_kind(tag.text.to_string(), tag.kind));
                        meta.count += 1;
                        meta.duration = meta.duration.add(&log.time_range().duration());
                        acc
                    },
                )
            },
        );

        let mut tag_summaries: Vec<TagSummary> = summary_map.values().cloned().fold(
            vec![],
            |mut list, mut tag_summary| {
                for rate in &tag_summary.get_rates(&context.rates) {
                    tag_summary.cost = Some(match tag_summary.cost {
                        Some(cost) => cost.add(&rate.cost_for_duration(&tag_summary.duration)),
                        None => rate.cost_for_duration(&tag_summary.duration),
                    })
                }
                list.push(tag_summary);
                list
            }
        );
        tag_summaries.sort_by(|a, b| b.duration.duration.cmp(&a.duration.duration));
        TagSummaries { tag_metas: tag_summaries }
    }

    pub(crate) fn with_filter(&self, filter: &Filter) -> Self {
        if filter.criterias.is_empty() {
            return self.clone();
        }
        Self {
            date: self.date.clone(),
            logs: self
                .logs
                .iter()
                .filter(|log| {
                    (|tokens: &Tokens| {
                        if tokens.len() == 0 {
                            return true;
                        }
                        for criteria in filter.criterias.iter() {
                            for token in tokens.to_vec() {
                                if token.kind == TokenKind::Prose {
                                    continue;
                                }
                                if criteria.is_satisfied_with(token) {
                                    return true;
                                }
                            }
                        }
                        false
                    })(log.description())
                })
            .cloned()
                .collect(),
        }
    }

    pub(crate) fn description(&self) -> Tokens {
        let mut parts: Vec<Vec<Token>> = Vec::new();

        let tags: Vec<Token> = self
            .logs
            .iter()
            .flat_map(|l| {
                l.description()
                    .clone()
                    .no_whitespace()
                    .by_kind(TokenKind::Tag)
                    .0
            })
        .unique()
            .intersperse(Token::prose(" ".to_string()))
            .collect();

        let tickets: Vec<Token> = self
            .logs
            .iter()
            .flat_map(|l| {
                l.description()
                    .clone()
                    .no_whitespace()
                    .by_kind(TokenKind::Ticket)
                    .0
            })
        .unique()
            .intersperse(Token::prose(" ".to_string()))
            .collect();

        let prose: Vec<Token> = self
            .logs
            .iter()
            .flat_map(|l| {
                l.description()
                    .clone()
                    .no_whitespace()
                    .by_kind(TokenKind::Prose)
                    .0
            })
        .intersperse(Token::prose(" ".to_string()))
            .collect();

        if !tags.is_empty() {
            parts.push(tags);
        }
        if !tickets.is_empty() {
            parts.push(tickets);
        }
        if !prose.is_empty() {
            parts.push(prose);
        }

        Tokens(
            parts
            .into_iter()
            .intersperse(vec![Token::prose(" ".to_string())])
            .flatten()
            .collect(),
        )
    }
}

pub struct TagSummaries {
    pub tag_metas: Vec<TagSummary>,
}

impl TagSummaries {
    pub fn iter(&self) -> Iter<'_, TagSummary> {
        self.tag_metas.iter()
    }
    pub fn len(&self) -> usize {
        self.tag_metas.len()
    }

    pub fn duration(&self) -> LogDuration {
        let minutes = self.iter().fold(0, |mut carry, tag_meta| {
            carry += tag_meta.duration.num_minutes();
            carry
        });
        LogDuration {
            duration: Duration::minutes(minutes),
        }
    }

    fn from_log_days(log_days: &Vec<LogDay>, context: &LogContext, tag: TokenKind) -> TagSummaries {
        let entry_map = log_days.iter().fold(
            HashMap::new(),
            |entry_map: HashMap<String, TagSummary>, day: &LogDay| {
                day.tag_summary(tag, context)
                    .iter()
                    .fold(entry_map, |mut entry_map, day_meta| {
                        let meta = entry_map
                            .entry(day_meta.tag.to_string())
                            .or_insert(TagSummary::from_tag_name_and_kind(day_meta.tag.clone(), day_meta.kind));
                        meta.merge(day_meta);
                        entry_map
                    })
            },
        );

        let mut tag_summaries: Vec<TagSummary> = entry_map.values().cloned().collect();
        tag_summaries.sort_by(|a, b| b.duration.duration.cmp(&a.duration.duration));
        TagSummaries { tag_metas: tag_summaries }
    }
}

#[derive(Clone)]
pub struct Money
{
    pub currency: Currency,
    pub amount: u64,
}

impl Money {
    pub fn new(currency: Currency, amount: u64) -> Self 
    {
        Self { currency, amount }
    }

    pub fn add(&self, money: &Money) -> Self
    {
        if self.currency != money.currency {
            panic!("Cannot perform addition with two different currencies: {} vs {}", self.currency, money.currency);
        }

        Self { currency: self.currency, amount: self.amount + money.amount }
    }

    fn major_units(&self) -> u64 {
        if self.amount == 0 {
            return 0;
        }

        match self.currency.subunit_fraction() {
            Some(fraction) => self.amount / (fraction as u64),
            None => 0,
        }
    }

    fn remaining_minor_units(&self) -> u64 {
        match self.currency.subunit_fraction() {
            Some(fraction) => self.amount - (fraction as u64 * self.major_units()),
            None => self.amount,
        }
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(
            format_args!(
                "{} {}.{:02}",
                self.currency.code(),
                self.major_units().to_formatted_string(&Locale::en),
                self.remaining_minor_units(),
            )
        )
    }
}

#[derive(Clone)]
pub struct TagSummary {
    pub tag: String,
    pub kind: TokenKind,
    pub duration: LogDuration,
    pub count: usize,
    pub cost: Option<Money>,
}

impl TagSummary {
    fn get_rates(&self, rates: &Rates) -> Vec<Rate> {
        match self.kind {
            TokenKind::Prose => vec![],
            TokenKind::Tag => rates.for_tag(&self.tag),
            TokenKind::Ticket => rates.for_ticket(&self.tag),
        }
    }

    fn from_tag_name_and_kind(tag: String, kind: TokenKind) -> Self
    {
        TagSummary{
            tag: tag,
            kind: kind,
            duration: LogDuration::from_minutes(0),
            count: 0,
            cost: None,
        }
    }

    fn merge(&mut self, new_tag_meta: &TagSummary) -> () {
        if self.tag != new_tag_meta.tag {
            panic!("Cannot merge tag meta with different tag name: {} vs {}", self.tag, new_tag_meta.tag);
        }
        if self.kind != new_tag_meta.kind {
            panic!("Cannot merge tag meta with different tag kind: {} vs {}", self.tag, new_tag_meta.tag);
        }

        self.duration = self.duration.add(&new_tag_meta.duration);
        self.count = self.count + new_tag_meta.count;
        self.cost = match &self.cost {
            Some(cost) => match &new_tag_meta.cost {
                Some(new_cost) => Some(cost.add(&new_cost)),
                None => None,
            }
            None => new_tag_meta.cost.clone(),
        };
    }
}

#[derive(Clone)]
pub struct LogEntry {
    time_range: TimeRangeView,
    desription: Tokens,
}

impl LogEntry {
    pub fn percentage_of_day(&self, day_total: i64) -> f64 {
        (self.time_range.duration().num_minutes() as f64 / day_total as f64) * 100.0
    }

    pub fn time_range(&self) -> &TimeRangeView {
        &self.time_range
    }

    pub(crate) fn description(&self) -> &Tokens {
        &self.desription
    }
}

#[derive(Clone)]
pub struct LogDate {
    now: NaiveDateTime,
    date: NaiveDate,
}

impl LogDate {
    pub fn is_today(&self) -> bool {
        self.date.year() == self.now.year()
            && self.date.month() == self.now.month()
            && self.date.day() == self.now.day()
    }

    pub(crate) fn to_verbose_string(&self) -> String {
        self.date.format("%A %e %B, %Y").to_string()
    }

    pub(crate) fn to_compact_string(&self) -> String {
        self.date.format("%d/%m/%Y").to_string()
    }
}

#[derive(Clone)]
pub struct LogDuration {
    duration: Duration,
}
impl LogDuration {
    pub fn num_minutes(&self) -> i64 {
        self.duration.num_minutes()
    }

    pub fn from_minutes(arg: i64) -> LogDuration {
        LogDuration {
            duration: Duration::minutes(arg),
        }
    }

    fn add(&self, duration: &LogDuration) -> LogDuration {
        LogDuration { duration: self.duration + duration.duration }
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

#[derive(Clone)]
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
    use super::Rate;
    use chrono::NaiveTime;

    use crate::parser::{
        filter::{TokenIs, UnaryOperator, UnaryOperatorKind},
        timesheet::{Date, Entry, Log, Time, TimeRange, Tokens},
    };

    #[test]
    fn log_view_percentage_of_day() {
        let l = LogEntry {
            time_range: TimeRangeView {
                start: NaiveTime::from_hms(0, 0, 0),
                end: NaiveTime::from_hms(12, 0, 0),
                ongoing: false,
            },
            desription: Tokens::from_prose("foo".to_string()),
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
            let view = LogDay::new(time, entry);
            assert_eq!("10:00:00-11:00:00", view.logs[0].time_range().to_string())
        }
    }

    #[test]
    fn test_minutes_by_weekday() {
        let mut entries = vec![];
        for day in 1..30 {
            entries.push(Entry {
                date: Date::from_ymd(2022, 01, day),
                logs: vec![Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::tag("foobar".to_string())]),
                }],
            });
        }

        let log_days = LogDays::new(entries);

        let minutes_by_weekday = log_days.minutes_by_weekday();
        println!("{:?}", minutes_by_weekday);
        assert_eq!(&("Mon", 600), minutes_by_weekday.first().unwrap());
    }

    #[test]
    fn test_filters_by_tag() {
        let days = LogDays::new(vec![Entry {
            date: Date::from_ymd(2022, 01, 1),
            logs: vec![
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::tag("foobar".to_string())]),
                },
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::tag("barfoo".to_string())]),
                },
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::ticket("FOO-1234".to_string())]),
                },
                Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::tag("foobar".to_string())]),
                },
            ],
        }]);
        assert_eq!(4, days.log_days[0].logs.len());

        let filtered = days.filter(&Filter::new(vec![Box::new(TokenIs {
            value: "foobar".to_string(),
            kind: TokenKind::Tag,
        })]));
        assert_eq!(2, filtered.log_days[0].logs.len());

        let filtered = days.filter(&Filter::new(vec![Box::new(TokenIs {
            value: "FOO-1234".to_string(),
            kind: TokenKind::Ticket,
        })]));
        assert_eq!(1, filtered.log_days[0].logs.len());
    }

    #[test]
    fn test_filters_not() {
        let days = LogDays::new(vec![Entry {
            date: Date::from_ymd(2022, 01, 1),
            logs: vec![Log {
                time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                description: Tokens::new(vec![
                    Token::prose("baz".to_string()),
                    Token::tag("foobar".to_string()),
                ]),
            }],
        }]);
        assert_eq!(1, days.log_days[0].logs.len());

        let filtered = days.filter(&Filter::new(vec![Box::new(UnaryOperator {
            kind: UnaryOperatorKind::Not,
            operand: Box::new(TokenIs {
                value: "foobar".to_string(),
                kind: TokenKind::Tag,
            }),
        })]));
        assert_eq!(0, filtered.log_days[0].logs.len());
    }

    #[test]
    fn test_tag_summary() {
        let days = LogDays::new(vec![Entry {
            date: Date::from_ymd(2022, 01, 1),
            logs: vec![Log {
                time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                description: Tokens::new(vec![
                    Token::prose("baz".to_string()),
                    Token::tag("foobar".to_string()),
                ]),
            }],
        }]);
        assert_eq!(1, days.log_days[0].tag_summary(TokenKind::Tag, &LogContext::new(days.clone(), Rates::from_rates(vec![
           Rate{
               ticket_prefix: None,
               tags: vec!["foobar".to_string()],
               rate: 100,
               currency: Currency::AFN
           }
        ]))).len());

        let filtered = days.filter(&Filter::new(vec![Box::new(UnaryOperator {
            kind: UnaryOperatorKind::Not,
            operand: Box::new(TokenIs {
                value: "foobar".to_string(),
                kind: TokenKind::Tag,
            }),
        })]));
        assert_eq!(0, filtered.log_days[0].logs.len());
    }

    #[test]
    fn log_day_applys_no_filter_with_no_criteria() {
        let day = LogDay::from_entry(
            Local::now().naive_local(),
            Entry {
                date: Date::from_ymd(2022, 01, 1),
                logs: vec![Log {
                    time: TimeRange::from_start_end(Time::from_hm(10, 0), Time::from_hm(12, 30)),
                    description: Tokens::new(vec![Token::tag("foobar".to_string())]),
                }],
            },
        );

        let day = day.with_filter(&Filter::new(vec![]));
        assert_eq!(1, day.logs.len());
    }
}
