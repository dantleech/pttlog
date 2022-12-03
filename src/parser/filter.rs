use std::fmt::Display;

use anyhow::{Error, Result};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::sequence;
use nom::{character::complete::multispace0, combinator::opt, multi::many0, sequence::tuple};

use crate::app::config::Config;

use super::token::{token, Token, TokenKind};

pub struct TokenIs {
    pub value: String,
    pub kind: TokenKind,
}

pub struct Not {
    pub criteria: Box<dyn Criteria>,
}

impl Criteria for Not {
    fn to_string(&self) -> String {
        format!("Not({})", self.criteria.to_string())
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        !self.criteria.is_satisfied_with(token)
    }
}

impl Criteria for TokenIs {
    fn to_string(&self) -> String {
        format!("{:?}({})", self.kind, self.value)
    }

    fn is_satisfied_with(&self, token: &Token) -> bool {
        self.value == token.text && self.kind == token.kind
    }
}

pub trait Criteria {
    fn to_string(&self) -> String;
    fn is_satisfied_with(&self, token: &Token) -> bool;
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

// TODO: Finish this
pub fn criteria<'a>(text: &'a str, _config: &Config) -> nom::IResult<&'a str, Box<dyn Criteria>> {
    let c = nom::combinator::map(tag("OR"), |_| -> Box<dyn Criteria> {
        Box::new(TokenIs {
            value: "foobar".to_string(),
            kind: TokenKind::Prose,
        })
    })(text);
    match c {
        Ok(ok) => Ok(ok),
        Err(err) => Err(err),
    }
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
