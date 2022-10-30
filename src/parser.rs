use chrono::NaiveDate;
use nom::{
    character::complete::{char, digit1, space0, not_line_ending, line_ending, multispace0}, multi::many0, combinator::{opt, map_res}, Parser
};
use nom::sequence;
use core::fmt::Debug;
use std::fmt::Display;

#[derive(Debug)]
pub struct Date {
    pub year: i16,
    pub month: i16,
    pub day: i16,
}

impl Date {
    pub fn sort_value(&self) -> i16 {
        return self.year + self.month + self.day;
    }
    pub fn to_string(&self) -> String {
        return format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub hour: i16,
    pub minute: i16,
}

impl Time {
    pub fn to_string(&self) -> String {
        return format!("{:02}:{:02}", self.hour, self.minute)
    }
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: Time,
    pub end: Option<Time>,
}

impl TimeRange {
    pub fn to_string(&self) -> String {
        if self.end.is_none() {
            return self.start.to_string();
        }

        return format!("{}-{}", self.start.to_string(), self.end.as_ref().unwrap().to_string())
    }

    fn duration(&self) -> i16 {
        if self.end.is_none() {
            return 0;
        }
        let end = self.end.unwrap();
        i16::from((
            (end.hour - self.start.hour) * 60
        ) + (end.minute - self.start.minute))
    }
}

#[derive(Debug)]
enum TokenKind {
    Prose,
    Tag,
}
impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

trait Token {
    fn kind(&self) -> TokenKind;
    fn as_string(&self) -> String;
}

impl Debug for dyn Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({}) = \"{}\"", self.kind(), self.as_string())
    }
}

#[derive(Debug)]
struct ProseToken {
    prose: String
}

impl Token for ProseToken {
    fn kind(&self) -> TokenKind {
        TokenKind::Prose
    }

    fn as_string(&self) -> String {
        format!("{}", self.prose)
    }
}

#[derive(Debug)]
pub struct Tokens(Vec<Box<dyn Token>>);

impl Tokens {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
    pub fn as_str(&self) {
    }
    pub fn to_string(&self) -> String {
        self.0.iter().fold("".to_string(), |acc, item| format!("{}{}", acc, item.as_string()))
        
    }

    fn new(tokens: Vec<Box<dyn Token>>) -> Tokens {
        Tokens(tokens)
    }
}

#[derive(Debug)]
pub struct Log {
    pub time: TimeRange,
    pub description: Tokens,
}

impl Log {
    pub fn set_duration(&mut self, end_time: &Time) {
        if self.time.end.is_some() {
            return;
        }
        self.time.end = Some(*end_time);
    }
    pub fn duration_as_string(&self) -> String {
        let quot = self.time.duration() / 60;
        let rem = self.time.duration() % 60;

        return format!("{}h{}m", quot, rem)

    }

    pub(crate) fn as_percentage(&self, duration_total: i16) -> f32 {
        return (f32::from(self.time.duration()) / f32::from(duration_total)) * 100.0;
    }
}

#[derive(Debug)]
pub struct Entry {
    pub date: Date,
    pub logs: Vec<Log>,
}

impl Entry {
    pub fn date_object(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date.to_string(), "%Y-%m-%d").expect("Could not parse date")
    }
    pub fn duration_total_as_string(&self) -> String {
        let quot = self.duration_total() / 60;
        let rem = self.duration_total() % 60;

        return format!("{}h{}m", quot, rem)
    }

    pub fn duration_total(&self) -> i16 {
        self.logs.iter().fold(0, |c,l| c + l.time.duration())
    }
}

#[derive(Debug)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

fn date_digits_i16(text: &str) -> nom::IResult<&str, i16> {
    map_res(digit1, str::parse)(text)
}

fn date(text: &str) -> nom::IResult<&str, Date>   {
    let date = sequence::tuple((
            date_digits_i16,
            char('-'),
            date_digits_i16,
            char('-'),
            date_digits_i16
            ))(text);

    match date {
        Ok(ok) => Ok((ok.0, Date{ year: (ok.1).0, month: (ok.1).2, day: (ok.1).4})),
        Err(err) => Err(err),
    }
}

fn time(text: &str) -> nom::IResult<&str, Time>   {
    let date = sequence::tuple((
            date_digits_i16,
            char(':'),
            date_digits_i16,
            ))(text);

    match date {
        Ok(ok) => Ok((ok.0, Time{ hour: (ok.1).0, minute: (ok.1).2})),
        Err(err) => Err(err),
    }
}
fn time_range(text: &str) -> nom::IResult<&str, TimeRange> {
    let time_range = sequence::tuple(
        (
            time,
            opt(sequence::pair(char('-'), time))
        )
    )(text);
    match time_range {
        Ok(ok) => {
            if (ok.1).1.is_some() {
                let end = (ok.1).1.unwrap();
                return Ok((ok.0, TimeRange{ start: (ok.1).0, end: Some(end.1)}));
            }
            Ok((ok.0, TimeRange{ start: (ok.1).0, end: None}))
        }
        Err(err) => Err(err),
    }
}


fn log(text: &str) -> nom::IResult<&str, Log>   {
    let entry = sequence::tuple((
            time_range,
            space0,
            not_line_ending
            ))(text);

    match entry {
        Ok(ok) => Ok((ok.0, Log{
            time: (ok.1).0,
            description: Tokens::new(vec![Box::new(ProseToken{prose: (ok.1).2.to_string()})]),
        })),
        Err(err) => Err(err),
    }
}

fn entry(text: &str) -> nom::IResult<&str, Entry>   {
    let entry = sequence::tuple((
            date,
            line_ending,
            multispace0,
            many0(
                sequence::tuple((log, opt(line_ending))).map(|t| t.0)
                )
            ))(text);

    match entry {
        Ok(ok) => Ok((ok.0, Entry{ date: (ok.1).0, logs: (ok.1).3 })),
        Err(err) => Err(err),
    }
}
pub fn parse(text: &str) -> nom::IResult<&str, Entries>   {
    let entry = sequence::tuple((
            multispace0,
            many0(
                sequence::tuple((entry, multispace0)).map(|t| t.0)
                )
            ))(text);

    match entry {
        Ok(ok) => {
            let mut entries = (ok.1).1;
            process_entries(&mut entries);
            Ok((ok.0, Entries{ entries}))
        }
        Err(err) => Err(err),
    }
}

fn process_entries(entries: &mut Vec<Entry>) {
    entries.sort_by_key(|e|e.date.sort_value());

    for entry in entries.iter_mut() {
        let mut last_log: Option<&mut Log> = None;
        for log in entry.logs.iter_mut() {
            if last_log.is_none() {
                last_log = Some(log);
                continue
            }
            last_log.unwrap().set_duration(&log.time.start);
            last_log = Some(log);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_date_digits() {
        assert_eq!(("", 2022), date_digits_i16("2022").unwrap());
        assert_eq!(("-10", 2022), date_digits_i16("2022-10").unwrap());
    }

    #[test]
    fn test_parse_date() {
        let (_, date) = date("2022-01-02").unwrap();
        assert_eq!(2022, date.year);
        assert_eq!(01, date.month);
        assert_eq!(02, date.day);
    }

    #[test]
    fn test_parse_date_invalid() {
        date("2022 -01-02").unwrap_err();
    }

    #[test]
    fn test_parse_log() {
        {
            let (_, entry) = log("10:00 Working on foo").unwrap();
            assert_eq!("Working on foo".to_string(), entry.description.to_string());
        }

        {
            let (_, entry) = log("09:00    Working on foo").unwrap();
            assert_eq!("Working on foo".to_string(), entry.description.to_string());
        }
    }

    #[test]
    fn test_parse_entry() {
        {
            let (_, entry) = entry("2022-01-01\n10:00 Working on foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description.to_string());
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo").unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description.to_string());
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo\n11:00 Working on bar").unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description.to_string());
            assert_eq!("Working on bar".to_string(), entry.logs[1].description.to_string());
        }
    }

    #[test]
    fn test_parse_entries() {
        {
            let (_, entries) = parse("2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entries.entries[0].date.to_string());
            assert_eq!("2022-02-02".to_string(), entries.entries[1].date.to_string());
        }
        {
            let (_, entries) = parse("2022-01-01\n\n\n10:00 Working on foo\n\n\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entries.entries[0].date.to_string());
            assert_eq!("2022-02-02".to_string(), entries.entries[1].date.to_string());
        }

        {
            let (_, entries) = parse("\n\n2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entries.entries[0].date.to_string());
        }
    }

    #[test]
    fn test_calculates_duration() {
        {
            let (_, entries) = parse("2022-01-01\n10:00 Working on foo\n11:00 Working on bar\n12:00 Doing something else").unwrap();
            assert_eq!("10:00", entries.entries[0].logs[0].time.start.to_string());
            assert_eq!("1h0m", entries.entries[0].logs[0].duration_as_string());
            assert_eq!("11:00", entries.entries[0].logs[1].time.start.to_string());
            assert_eq!("12:00", entries.entries[0].logs[2].time.start.to_string());
        }
    }

    #[test]
    fn test_sorts_entries_by_date_asc() {
        {
            let (_, entries) = parse("2022-01-01\n2021-01-01\n").unwrap();
            assert_eq!(2, entries.entries.len());
            assert_eq!("2021-01-01".to_string(), entries.entries[0].date.to_string());
            assert_eq!("2022-01-01".to_string(), entries.entries[1].date.to_string());
        }
    }

    #[test]
    fn test_parses_time_range() {
        {
            let (_, entries) = parse("2022-01-01\n20:00-21:00").unwrap();
            assert_eq!(1, entries.entries.len());
            assert_eq!("20:00-21:00".to_string(), entries.entries[0].logs[0].time.to_string());
        }
    }
}
