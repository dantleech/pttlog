use std::fmt::Display;

use anyhow::{Error, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::map;
use nom::sequence;
use nom::{character::complete::multispace0, combinator::opt, multi::many0, sequence::tuple};

use crate::app::config::Config;

use super::token::{token, Token, TokenKind};

pub trait Criteria {
    fn to_string(&self) -> String;
    fn is_satisfied_with(&self, token: &Token) -> bool;
}

#[derive(Debug)]
pub enum BinaryOperatorKind {
    And,
    Or,
    Unknown,
}

pub struct BinaryOperator {
    pub kind: BinaryOperatorKind,
    pub left: Box<dyn Criteria>,
    pub right: Box<dyn Criteria>,
}

impl Criteria for BinaryOperator {
    fn to_string(&self) -> String {
        format!("{:?}", self.kind)
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        return true;
    }
}

pub struct TokenIs {
    pub value: String,
    pub kind: TokenKind,
}

pub struct Not {
    pub criteria: Box<dyn Criteria>,
}

impl Criteria for TokenIs {
    fn to_string(&self) -> String {
        format!("{:?}({})", self.kind, self.value)
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        self.value == token.text && self.kind == token.kind
    }
}

impl Criteria for Not {
    fn to_string(&self) -> String {
        format!("Not({})", self.criteria.to_string())
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        !self.criteria.is_satisfied_with(token)
    }
}

pub struct Filter {
    pub criterias: Vec<Box<dyn Criteria>>,
}

impl Filter {
    pub fn new(criterias: Vec<Box<dyn Criteria>>) -> Self {
        Filter { criterias }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strings: Vec<String> = self
            .criterias
            .iter()
            .map(|criteria| criteria.to_string())
            .collect();

        f.write_str(&strings.join(" OR "))?;
        Ok(())
    }
}

fn binary_operator<'a>(
    text: &'a str,
    config: &Config,
) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    map(
        sequence::tuple((
            alt((tag("OR"), tag("AND"))),
            |text| criteria(text, config),
            |text| criteria(text, config),
        )),
        |res| -> Box<dyn Criteria> {
            Box::new(BinaryOperator{
                kind: match res.0 {
                        "OR" => BinaryOperatorKind::Or,
                        "AND" => BinaryOperatorKind::And,
                        _ => BinaryOperatorKind::Unknown,
                },
                left: res.1,
                right: res.2,
            })
        }
    )(text)
}

fn token_match<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    map(|text| token(text, config), |res| -> Box<dyn Criteria> {
        Box::new(TokenIs{
            value: res.text,
            kind: res.kind,
        })
    })(text)
}

fn criteria<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    alt((
        |text| token_match(text, config),
        |text| binary_operator(text, config),
    ))(text)
}

pub fn parse_filter<'a>(text: &'a str, config: &Config) -> Result<Filter> {
    let tokens = many0(tuple((|input| criteria(input, config), multispace0)))(text);

    match tokens {
        Ok(ok) => Ok(Filter {
            criterias: ok.1.into_iter().map(|criteria| criteria.0).collect(),
        }),
        Err(err) => Err(Error::msg(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::app::config::Project;

    use super::*;

    #[test]
    fn test_parse_filter() {
        let parsed = parse_filter("@foobar", &Config::empty()).unwrap();
        assert_eq!(1, parsed.criterias.len());
        assert_eq!("Tag(foobar)", parsed.criterias[0].to_string())
    }

    #[test]
    fn test_parse_many_tags() {
        let parsed = parse_filter("@foobar ~@barfoo", &Config::empty()).unwrap();
        assert_eq!(2, parsed.criterias.len());
        assert_eq!("Tag(foobar) OR Not(Tag(barfoo))", parsed.to_string())
    }

    #[test]
    fn test_parse_many_tags_and_tickets() {
        let config = Config {
            projects: vec![Project {
                name: "myproject".to_string(),
                ticket_prefix: "PROJECT-".to_string(),
                tags: vec![],
            }],
        };
        let parsed = parse_filter("@foobar ~PROJECT-5 PROJECT-12", &config).unwrap();
        assert_eq!(3, parsed.criterias.len());
        assert_eq!(
            "Tag(foobar) OR Not(Ticket(PROJECT-5)) OR Ticket(PROJECT-12)",
            parsed.to_string()
        )
    }

    #[test]
    fn test_or() {
        let config = Config::empty();
        let parsed = parse_filter("OR @foobar OR ~PROJECT-5 PROJECT-12", &config).unwrap();
        assert_eq!(2, parsed.criterias.len());
        assert_eq!(
            "Tag(foobar) OR Not(Ticket(PROJECT-5)) OR Ticket(PROJECT-12)",
            parsed.to_string()
        )
    }
}
