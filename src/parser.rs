use chrono::{NaiveDate};
use nom::{
    character::complete::{char, digit1, space0, not_line_ending, line_ending, multispace0}, multi::many0, combinator::{opt, map_res}, Parser
};
use nom::sequence;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: i16,
    pub minute: i16,
}

impl Time {
    pub fn to_string(&self) -> String {
        return format!("{:02}:{:02}", self.hour, self.minute)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Log {
    pub time: Time,
    pub description: String,
    pub duration: i16,
}

impl Log {
    pub fn set_duration(&mut self, end_time: &Time) {
        self.duration = i16::from((
            (end_time.hour - self.time.hour) * 60
        ) + (end_time.minute - self.time.minute))
    }
    pub fn duration_as_string(&self) -> String {
        let quot = self.duration / 60;
        let rem = self.duration % 60;

        return format!("{}h{}m", quot, rem)

    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        self.logs.iter().fold(0, |c,l| c + l.duration)
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

fn log(text: &str) -> nom::IResult<&str, Log>   {
    let entry = sequence::tuple((
            time,
            space0,
            not_line_ending
            ))(text);

    match entry {
        Ok(ok) => Ok((ok.0, Log{
            time: (ok.1).0,
            description: (ok.1).2.to_string(),
            duration: 0,
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
            last_log.unwrap().set_duration(&log.time);
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
            assert_eq!("Working on foo".to_string(), entry.description);
        }

        {
            let (_, entry) = log("09:00    Working on foo").unwrap();
            assert_eq!("Working on foo".to_string(), entry.description);
        }
    }

    #[test]
    fn test_parse_entry() {
        {
            let (_, entry) = entry("2022-01-01\n10:00 Working on foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo").unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo\n11:00 Working on bar").unwrap();
            assert_eq!("2022-01-01", entry.date.to_string());
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
            assert_eq!("Working on bar".to_string(), entry.logs[1].description);
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
            assert_eq!("10:00", entries.entries[0].logs[0].time.to_string());
            assert_eq!("1h0m", entries.entries[0].logs[0].duration_as_string());
            assert_eq!("11:00", entries.entries[0].logs[1].time.to_string());
            assert_eq!("12:00", entries.entries[0].logs[2].time.to_string());
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
}
