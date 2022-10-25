use nom::{
    character::complete::{char, digit1, space0, not_line_ending, line_ending, multispace0}, multi::many0, combinator::{opt, map_res}, Parser
};
use nom::sequence;

#[derive(Debug)]
pub struct Date {
    pub y: i16,
    pub m: i8,
    pub d: i8,
}

impl Date {
    pub fn to_string(&self) -> String {
        return format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

#[derive(Debug)]
pub struct Time {
    pub hour: i8,
    pub minute: i8,
}

impl Time {
    pub fn to_string(&self) -> String {
        return format!("{:02}:{:02}", self.hour, self.minute)
    }
}

#[derive(Debug)]
pub struct Log {
    pub time: Time,
    pub description: String,
    pub duration: i8,
}

#[derive(Debug)]
pub struct Entry {
    pub date: Date,
    pub logs: Vec<Log>,
}

#[derive(Debug)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

fn date_digits_i8(text: &str) -> nom::IResult<&str, i8> {
    map_res(digit1, str::parse)(text)
}
fn date_digits_i16(text: &str) -> nom::IResult<&str, i16> {
    map_res(digit1, str::parse)(text)
}

fn date(text: &str) -> nom::IResult<&str, Date>   {
    let date = sequence::tuple((
        date_digits_i16,
        char('-'),
        date_digits_i8,
        char('-'),
        date_digits_i8
    ))(text);

    match date {
        Ok(ok) => Ok((ok.0, Date{ y: (ok.1).0, m: (ok.1).2, d: (ok.1).4})),
        Err(err) => Err(err),
    }
}

fn time(text: &str) -> nom::IResult<&str, Time>   {
    let date = sequence::tuple((
        date_digits_i8,
        char(':'),
        date_digits_i8,
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
        Ok(ok) => Ok((ok.0, Entries{ entries: process_entries((ok.1).1) })),
        Err(err) => Err(err),
    }
}

fn process_entries(entries: Vec<Entry>) -> Vec<Entry> {

    for entry in entries.iter() {
        let mut last_log: Option<&Log> = None;
        for log in entry.logs.iter() {
            if last_log.is_none() {
                last_log = Some(log);
                continue
            }
        }
    }
    return entries
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
        assert_eq!(2022, date.y);
        assert_eq!(01, date.m);
        assert_eq!(02, date.d);
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
        }

        {
            let (_, entries) = parse("\n\n2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022-01-01".to_string(), entries.entries[0].date.to_string());
        }
    }

    #[test]
    fn test_parse_logs() {
        {
            let (_, entries) = parse("2022-01-01\n10:00 Working on foo\n11:00 Working on bar\n12:00 Doing something else").unwrap();
            assert_eq!("10:00", entries.entries[0].logs[0].time.to_string());
            assert_eq!(1, entries.entries[0].logs[0].duration);
            assert_eq!("11:00", entries.entries[0].logs[1].time.to_string());
            assert_eq!("12:00", entries.entries[0].logs[2].time.to_string());
        }
    }
}
