use chrono::Datelike;
use chrono::{NaiveDate, NaiveTime, Timelike};
use core::fmt::Debug;
use nom::sequence;
use nom::{
    character::complete::{char, digit1, line_ending, multispace0, space0},
    combinator::{map_res, opt},
    multi::many0,
    Parser,
};

use crate::app::config::Config;

use super::token::{token, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Date {
    date: NaiveDate,
}

impl Date {
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Date {
        Date {
            date: NaiveDate::from_ymd(year, month, day),
        }
    }
}

impl Date {
    pub fn year(&self) -> i32 {
        self.date.year()
    }
    pub fn month(&self) -> u32 {
        self.date.month()
    }
    pub fn day(&self) -> u32 {
        self.date.day()
    }
}

impl Date {
    pub fn sort_value(&self) -> i32 {
        format!(
            "{:04}{:02}{:02}",
            self.date.year(),
            self.date.month(),
            self.date.day()
        )
        .parse::<i32>()
        .unwrap()
    }
    pub fn to_string(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}",
            self.date.year(),
            self.date.month(),
            self.date.day()
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    time: NaiveTime,
}

impl Time {
    pub(crate) fn time(&self) -> NaiveTime {
        self.time
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}
impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Time {
    /// # Example
    ///
    /// ```rust
    /// use pttlogger::parser::Time;
    ///
    /// let t = Time::from_hm(10,10);
    /// assert_eq!("10:10", t.to_string());
    /// ```
    ///
    /// ```rust
    /// use pttlogger::parser::Time;
    ///
    /// let t = Time::from_hm(24,10);
    /// assert_eq!("00:10", t.to_string());
    /// ```
    pub fn from_hm(h: u32, m: u32) -> Time {
        Time {
            time: NaiveTime::from_hms(h % 24, m % 60, 0),
        }
    }
    pub fn hour(&self) -> u32 {
        self.time.hour()
    }
    pub fn minute(&self) -> u32 {
        self.time.minute()
    }
    pub fn to_string(&self) -> String {
        self.time.format("%H:%M").to_string()
    }
}

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: Time,
    pub end: Option<Time>,
}

impl TimeRange {
    pub fn from_start_end(start: Time, end: Time) -> TimeRange {
        TimeRange {
            start,
            end: Some(end),
        }
    }
    pub fn from_start(start: Time) -> TimeRange {
        TimeRange { start, end: None }
    }

    pub fn to_string(&self) -> String {
        if self.end.is_none() {
            return self.start.to_string();
        }

        format!(
            "{}-{}",
            self.start.to_string(),
            self.end.as_ref().unwrap().to_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Tokens(pub Vec<Token>);

impl Tokens {
    pub fn from_prose(prose: String) -> Tokens {
        Tokens(vec![Token {
            kind: TokenKind::Prose,
            text: prose,
            whitespace: "".to_string(),
        }])
    }
    pub fn to_vec(&self) -> &Vec<Token> {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> &Token {
        assert!(
            !self.0.is_empty(),
            "Cannot get first token when tokens are empty"
        );
        self.0.first().unwrap()
    }

    pub fn at(&self, index: usize) -> &Token {
        &self.0[index]
    }

    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
    pub fn to_string(&self) -> String {
        self.0.iter().fold("".to_string(), |acc, item| {
            format!("{}{}", acc, item.to_string())
        })
    }

    pub fn new(tokens: Vec<Token>) -> Tokens {
        Tokens(tokens)
    }

    pub fn tags(&self) -> Vec<&Token> {
        self.0.iter().filter(|t| t.kind == TokenKind::Tag).collect()
    }

    pub(crate) fn by_kind(&self, kind: TokenKind) -> Vec<&Token> {
        self.0.iter().filter(|t| t.kind == kind).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub time: TimeRange,
    pub description: Tokens,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub date: Date,
    pub logs: Vec<Log>,
}

impl Entry {
    pub fn date_object(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date.to_string(), "%Y-%m-%d").expect("Could not parse date")
    }
    pub(crate) fn placeholder() -> Entry {
        Entry {
            date: Date {
                date: NaiveDate::from_ymd_opt(2015, 1, 1).unwrap(),
            },
            logs: vec![Log {
                time: TimeRange {
                    start: Time::from_hm(7, 28),
                    end: Some(Time::from_hm(8, 28)),
                },
                description: Tokens::from_prose(
                    "Marty! this plain text time sheet is empty Marty!".to_string(),
                ),
            }],
        }
    }
}

#[derive(Debug)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

fn date_digits_i32(text: &str) -> nom::IResult<&str, i32> {
    map_res(digit1, str::parse)(text)
}

fn date(text: &str) -> nom::IResult<&str, Date> {
    let date = sequence::tuple((
        date_digits_i32,
        char('-'),
        date_digits_i32,
        char('-'),
        date_digits_i32,
    ))(text);

    match date {
        Ok(ok) => Ok((
            ok.0,
            Date {
                date: NaiveDate::from_ymd_opt(
                    (ok.1).0,
                    (ok.1).2.try_into().unwrap(),
                    (ok.1).4.try_into().unwrap(),
                )
                .unwrap(),
            },
        )),
        Err(err) => Err(err),
    }
}

fn time(text: &str) -> nom::IResult<&str, Time> {
    let time = sequence::tuple((date_digits_i32, char(':'), date_digits_i32))(text);

    match time {
        Ok(ok) => Ok((
            ok.0,
            Time::from_hm(
                (ok.1).0.try_into().unwrap(),
                u32::try_from((ok.1).2).unwrap(),
            ),
        )),
        Err(err) => Err(err),
    }
}

fn time_range(text: &str) -> nom::IResult<&str, TimeRange> {
    let time_range = sequence::tuple((time, opt(sequence::pair(char('-'), time))))(text);
    match time_range {
        Ok(ok) => {
            if (ok.1).1.is_some() {
                let end = (ok.1).1.unwrap();
                return Ok((
                    ok.0,
                    TimeRange {
                        start: (ok.1).0,
                        end: Some(end.1),
                    },
                ));
            }
            Ok((
                ok.0,
                TimeRange {
                    start: (ok.1).0,
                    end: None,
                },
            ))
        }
        Err(err) => Err(err),
    }
}

fn log<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Log> {
    let entry = sequence::tuple((time_range, space0, many0(|input| token(input, config))))(text);

    match entry {
        Ok(ok) => Ok((
            ok.0,
            Log {
                time: (ok.1).0,
                description: Tokens::new((ok.1).2),
            },
        )),
        Err(err) => Err(err),
    }
}

fn entry<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Entry> {
    let entry = sequence::tuple((
        date,
        line_ending,
        multispace0,
        many0(sequence::tuple((|input| log(input, config), opt(line_ending))).map(|t| t.0)),
    ))(text);

    match entry {
        Ok(ok) => Ok((
            ok.0,
            Entry {
                date: (ok.1).0,
                logs: (ok.1).3,
            },
        )),
        Err(err) => Err(err),
    }
}
pub fn parse_entry<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Entries> {
    let entry = sequence::tuple((
        multispace0,
        many0(sequence::tuple((|input| entry(input, config), multispace0)).map(|t| t.0)),
    ))(text);

    match entry {
        Ok(ok) => {
            let mut entries = (ok.1).1;
            process_entries(&mut entries);
            Ok((ok.0, Entries { entries }))
        }
        Err(err) => Err(err),
    }
}

fn process_entries(entries: &mut Vec<Entry>) {
    entries.sort_by_key(|e| e.date.sort_value());
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::app::config::Project;

    use super::*;
    #[test]
    fn test_parse_date_digits() {
        assert_eq!(("", 2022), date_digits_i32("2022").unwrap());
        assert_eq!(("-10", 2022), date_digits_i32("2022-10").unwrap());
    }

    #[test]
    fn test_parse_date() {
        let (_, date) = date("2022-01-02").unwrap();
        assert_eq!(2022, date.year());
        assert_eq!(01, date.month());
        assert_eq!(02, date.day());
    }

    #[test]
    fn test_parse_date_invalid() {
        date("2022 -01-02").unwrap_err();
    }

    #[test]
    fn test_parse_log() {
        {
            let (_, entry) = log("10:00 Working on foo", &Config::empty()).unwrap();
            assert_eq!("Working on foo".to_string(), entry.description.to_string());
        }

        {
            let (_, entry) = log("09:00    Working on foo", &Config::empty()).unwrap();
            assert_eq!("Working on foo".to_string(), entry.description.to_string());
        }
    }

    #[test]
    fn test_parse_entry() {
        {
            let (_, entry) = entry("2022-01-01\n10:00 Working on foo", &Config::empty()).unwrap();
            assert_eq!("2022-01-01".to_string(), entry.date.to_string());
            assert_eq!(
                "Working on foo".to_string(),
                entry.logs[0].description.to_string()
            );
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo", &Config::empty()).unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!(
                "Working on foo".to_string(),
                entry.logs[0].description.to_string()
            );
        }

        {
            let (_, entry) = entry(
                "2022-01-01\n\n10:00 Working on foo\n11:00 Working on bar",
                &Config::empty(),
            )
            .unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!(
                "Working on foo".to_string(),
                entry.logs[0].description.to_string()
            );
            assert_eq!(
                "Working on bar".to_string(),
                entry.logs[1].description.to_string()
            );
        }
    }

    #[test]
    fn test_parse_entries() {
        {
            let (_, entries) = parse_entry(
                "2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo",
                &Config::empty(),
            )
            .unwrap();
            assert_eq!(2, entries.entries.len());
            assert_eq!(
                "2022-01-01".to_string(),
                entries.entries[0].date.to_string()
            );
            assert_eq!(
                "2022-02-02".to_string(),
                entries.entries[1].date.to_string()
            );
        }
        {
            let (_, entries) = parse_entry(
                "2022-01-01\n\n\n10:00 Working on foo\n\n\n2022-02-02\n11:00 Foo",
                &Config::empty(),
            )
            .unwrap();
            assert_eq!(
                "2022-01-01".to_string(),
                entries.entries[0].date.to_string()
            );
            assert_eq!(
                "2022-02-02".to_string(),
                entries.entries[1].date.to_string()
            );
        }

        {
            let (_, entries) = parse_entry(
                "\n\n2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo",
                &Config::empty(),
            )
            .unwrap();
            assert_eq!(
                "2022-01-01".to_string(),
                entries.entries[0].date.to_string()
            );
        }
    }

    #[test]
    fn test_sorts_entries_by_date_asc() {
        {
            let (_, entries) = parse_entry("2022-01-01\n2021-01-01\n", &Config::empty()).unwrap();
            assert_eq!(2, entries.entries.len());
            assert_eq!(
                "2021-01-01".to_string(),
                entries.entries[0].date.to_string()
            );
            assert_eq!(
                "2022-01-01".to_string(),
                entries.entries[1].date.to_string()
            );
        }
        {
            let (_, entries) = parse_entry("2022-01-31\n2022-02-01\n", &Config::empty()).unwrap();
            assert_eq!(2, entries.entries.len());
            assert_eq!(
                "2022-01-31".to_string(),
                entries.entries[0].date.to_string()
            );
            assert_eq!(
                "2022-02-01".to_string(),
                entries.entries[1].date.to_string()
            );
        }
    }

    #[test]
    fn test_parses_time_range() {
        {
            let (_, entries) = parse_entry("2022-01-01\n20:00-21:00", &Config::empty()).unwrap();
            assert_eq!(1, entries.entries.len());
            assert_eq!(
                "20:00-21:00".to_string(),
                entries.entries[0].logs[0].time.to_string()
            );
        }
    }

    #[test]
    fn test_parse_tag() {
        {
            let (_, entries) =
                parse_entry("2022-01-01\n20:00-21:00 Foobar @foobar", &Config::empty()).unwrap();
            assert_eq!(1, entries.entries.len());
            assert_eq!(
                "Foobar ".to_string(),
                entries.entries[0].logs[0]
                    .description
                    .first()
                    .to_string()
            );
            assert_eq!(
                "foobar".to_string(),
                entries.entries[0].logs[0].description.at(1).deref().text
            );
            assert_eq!(
                TokenKind::Tag,
                entries.entries[0].logs[0].description.at(1).deref().kind
            );
        }
        {
            let (_, entries) = parse_entry(
                "2022-01-01\n20:00-21:00 Foobar @foobar barfoo",
                &Config::empty(),
            )
            .unwrap();
            assert_eq!(1, entries.entries.len());
            assert_eq!(
                "foobar".to_string(),
                entries.entries[0].logs[0].description.at(1).deref().text
            );
            assert_eq!(
                TokenKind::Tag,
                entries.entries[0].logs[0].description.at(1).deref().kind
            );

            assert_eq!(
                "barfoo".to_string(),
                entries.entries[0].logs[0].description.at(2).deref().text
            );
        }
    }

    #[test]
    fn test_parse_ticket() {
        {
            let config = Config {
                projects: vec![
                    Project {
                        name: "myproject".to_string(),
                        ticket_prefix: "PROJECT-".to_string(),
                        tags: vec![],
                    },
                    Project {
                        name: "myproject".to_string(),
                        ticket_prefix: "BAR-".to_string(),
                        tags: vec![],
                    },
                ],
            };
            let (_, entries) = parse_entry(
                "2022-01-01\n20:00-21:00 BAR-12 BAZ-15 PROJECT-1 @foobar",
                &config,
            )
            .unwrap();
            assert_eq!(1, entries.entries.len());
            let description = &entries.entries[0].logs[0].description;

            assert_eq!(TokenKind::Ticket, description.at(0).kind);
            assert_eq!("BAR-12".to_string(), description.at(0).text());

            assert_eq!(TokenKind::Prose, description.at(1).kind);
            assert_eq!("BAZ-15".to_string(), description.at(1).text());

            assert_eq!(TokenKind::Ticket, description.at(2).kind);
            assert_eq!("PROJECT-1".to_string(), description.at(2).text());
        }
    }

    #[test]
    fn test_parse_tag_with_space() {
        let (_, entries) = parse_entry(
            "2022-01-01\n20:00 @foobar \n2022-02-02\n20:00 @barfoo\n",
            &Config::empty(),
        )
        .unwrap();
        println!("{:?}", entries);
        assert_eq!(2, entries.entries.len());
        assert_eq!(
            "barfoo".to_string(),
            entries.entries[1].logs[0].description.at(0).deref().text
        );
    }

    #[test]
    fn test_parse_tag_with_space_and_subsequent_token() {
        let (_, entries) =
            parse_entry("2022-01-01\n20:00 @foobar barfoo", &Config::empty()).unwrap();
        println!("{:?}", entries);
        assert_eq!(1, entries.entries.len());
        let description = &entries.entries[0].logs[0].description;
        assert_eq!(2, description.len());
        assert_eq!("foobar ".to_string(), description.at(0).to_string());
        assert_eq!("barfoo".to_string(), description.at(1).deref().text);
    }
}
