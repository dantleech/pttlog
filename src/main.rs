use nom::{
    character::{complete::{char, digit1, space0, not_line_ending, line_ending, multispace0}}, multi::many0, combinator::opt, Parser
};
use nom::sequence;

fn main() {
    println!("Hello, world!");
}

#[derive(Debug)]
pub struct Date {
    pub y: String,
    pub m: String,
    pub d: String,
}

#[derive(Debug)]
pub struct Time {
    pub hour: String,
    pub minute: String,
}

#[derive(Debug)]
pub struct Log {
    pub time: Time,
    pub description: String,
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

fn date_digits(text: &str) -> nom::IResult<&str, String> {
    let result = digit1(text);
    match result {
        Ok(ok) => Ok((ok.0, ok.1.to_string())),
        Err(err) => Err(err),
    }
}

pub fn date(text: &str) -> nom::IResult<&str, Date>   {
    let date = sequence::tuple((
        date_digits,
        char('-'),
        date_digits,
        char('-'),
        date_digits
    ))(text);

    match date {
        Ok(ok) => Ok((ok.0, Date{ y: (ok.1).0, m: (ok.1).2, d: (ok.1).4})),
        Err(err) => Err(err),
    }
}

pub fn time(text: &str) -> nom::IResult<&str, Time>   {
    let date = sequence::tuple((
        date_digits,
        char(':'),
        date_digits,
    ))(text);

    match date {
        Ok(ok) => Ok((ok.0, Time{ hour: (ok.1).0, minute: (ok.1).2})),
        Err(err) => Err(err),
    }
}

pub fn log(text: &str) -> nom::IResult<&str, Log>   {
    let entry = sequence::tuple((
        time,
        space0,
        not_line_ending
    ))(text);

    match entry {
        Ok(ok) => Ok((ok.0, Log{ time: (ok.1).0, description: (ok.1).2.to_string()})),
        Err(err) => Err(err),
    }
}

pub fn entry(text: &str) -> nom::IResult<&str, Entry>   {
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
pub fn entries(text: &str) -> nom::IResult<&str, Entries>   {
    let entry = sequence::tuple((
            multispace0,
            many0(
                sequence::tuple((entry, multispace0)).map(|t| t.0)
            )
    ))(text);

    match entry {
        Ok(ok) => Ok((ok.0, Entries{ entries: (ok.1).1 })),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_date_digits() {
        assert_eq!(("", "2022".to_string()), date_digits("2022").unwrap());
        assert_eq!(("-10", "2022".to_string()), date_digits("2022-10").unwrap());
    }

    #[test]
    fn test_parse_date() {
        let (_, date) = date("2022-01-02").unwrap();
        assert_eq!("2022", date.y);
        assert_eq!("01", date.m);
        assert_eq!("02", date.d);
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
            assert_eq!("2022".to_string(), entry.date.y);
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo").unwrap();
            assert_eq!("2022".to_string(), entry.date.y);
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
        }

        {
            let (_, entry) = entry("2022-01-01\n\n10:00 Working on foo\n11:00 Working on bar").unwrap();
            assert_eq!("2022".to_string(), entry.date.y);
            assert_eq!("Working on foo".to_string(), entry.logs[0].description);
            assert_eq!("Working on bar".to_string(), entry.logs[1].description);
        }
    }

    #[test]
    fn test_parse_entries() {
        {
            let (_, entries) = entries("2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022".to_string(), entries.entries[0].date.y);
        }

        {
            let (_, entries) = entries("\n\n2022-01-01\n10:00 Working on foo\n2022-02-02\n11:00 Foo").unwrap();
            assert_eq!("2022".to_string(), entries.entries[0].date.y);
            assert_eq!("02".to_string(), entries.entries[1].date.m);
        }
    }
}
