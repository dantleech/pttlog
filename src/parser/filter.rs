use std::fmt::Display;

use anyhow::{Error, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::multispace1;
use nom::combinator::map;
use nom::sequence;
use nom::{character::complete::multispace0, multi::many0, sequence::tuple};

use crate::app::config::Config;

use super::token::{Token, TokenKind, ticket_token, tag_token};

pub trait Criteria {
    fn to_string(&self) -> String;
    fn is_satisfied_with(&self, token: &Token) -> bool;
}

#[derive(Debug)]
pub enum UnaryOperatorKind {
    Not,
    Unknown,
}

pub struct UnaryOperator {
    pub kind: UnaryOperatorKind,
    pub operand: Box<dyn Criteria>,
}

impl Criteria for UnaryOperator {
    fn to_string(&self) -> String {
        format!("{:?}({})", self.kind, self.operand.to_string())
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        match self.kind {
            UnaryOperatorKind::Not => !self.operand.is_satisfied_with(token),
            UnaryOperatorKind::Unknown => panic!("Unknown unary operator (should not happen)"),
        }
        
    }
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
        format!("{:?}({}, {})", self.kind, self.left.to_string(), self.right.to_string())
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        match self.kind {
            BinaryOperatorKind::And => self.left.is_satisfied_with(token) && self.right.is_satisfied_with(token),
            BinaryOperatorKind::Or => self.left.is_satisfied_with(token) || self.right.is_satisfied_with(token),
            BinaryOperatorKind::Unknown => panic!("Unknown binary operator (should not happen)"),
        }
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

        f.write_str(&strings.join(" "))?;
        Ok(())
    }
}

fn binary_operator<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    map(
        sequence::tuple((
            alt((tag("OR"), tag("AND"))),
            multispace1,
            |text| criteria(text, config),
            |text| criteria(text, config),
            multispace0,
        )),
        |res| -> Box<dyn Criteria> {
            Box::new(BinaryOperator {
                kind: match res.0 {
                    "OR" => BinaryOperatorKind::Or,
                    "AND" => BinaryOperatorKind::And,
                    _ => BinaryOperatorKind::Unknown,
                },
                left: res.2,
                right: res.3,
            })
        },
    )(text)
}

fn unary_operator<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    map(
        sequence::tuple((
            tag("NOT"),
            multispace0,
            |text| criteria(text, config),
            multispace0,
        )),
        |res| -> Box<dyn Criteria> {
            Box::new(UnaryOperator {
                kind: match res.0 {
                    "NOT" => UnaryOperatorKind::Not,
                    _ => UnaryOperatorKind::Unknown,
                },
                operand: res.2,
            })
        },
    )(text)
}

fn token_match<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    map(
        sequence::tuple((
            multispace0,
            alt((
                |text| ticket_token(text, config),
                |text| tag_token(text),
            )),
            multispace0,
        )),
        |res| -> Box<dyn Criteria> {
            Box::new(TokenIs {
                value: res.1.text,
                kind: res.1.kind,
            })
        },
    )(text)
}

fn criteria<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    alt((
        |text| unary_operator(text, config),
        |text| binary_operator(text, config),
        |text| token_match(text, config),
    ))(text)
}

pub fn parse_filter(text: &str, config: &Config) -> Result<Filter> {
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
    fn test_parse_tag() {
        let parsed = parse_filter("@foobar", &Config::empty()).unwrap();
        assert_eq!(1, parsed.criterias.len());
        assert_eq!("Tag(foobar)", parsed.criterias[0].to_string())
    }

    #[test]
    fn test_parse_ticket() {
        let config = Config {
            projects: vec![Project {
                name: "myproject".to_string(),
                ticket_prefix: "PROJECT-".to_string(),
                tags: vec![],
            }],
        };
        let parsed = parse_filter("PROJECT-123", &config).unwrap();
        assert_eq!(1, parsed.criterias.len());
        assert_eq!("Ticket(PROJECT-123)", parsed.criterias[0].to_string())
    }

    #[test]
    fn test_parse_many_tags() {
        let parsed = parse_filter("@foobar NOT @barfoo", &Config::empty()).unwrap();
        assert_eq!(2, parsed.criterias.len());
        assert_eq!("Tag(foobar) Not(Tag(barfoo))", parsed.to_string())
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
        let parsed = parse_filter("@foobar NOT PROJECT-5 PROJECT-12", &config).unwrap();
        assert_eq!(3, parsed.criterias.len());
        assert_eq!(
            "Tag(foobar) Not(Ticket(PROJECT-5)) Ticket(PROJECT-12)",
            parsed.to_string()
        )
    }

    #[test]
    fn test_or() {
        let config = Config::empty();
        let parsed = parse_filter("OR @foobar OR NOT @bazboo @bag", &config).unwrap();
        assert_eq!(
            "Or(Tag(foobar), Or(Not(Tag(bazboo)), Tag(bag)))",
            parsed.to_string()
        )
    }

    #[test]
    fn test_and() {
        let config = Config::empty();
        let parsed = parse_filter("AND @foobar AND NOT @bazboo @bag", &config).unwrap();
        assert_eq!(
            "And(Tag(foobar), And(Not(Tag(bazboo)), Tag(bag)))",
            parsed.to_string()
        )
    }
}
