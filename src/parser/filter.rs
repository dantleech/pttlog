use std::{boxed, fmt::Display};

use anyhow::{Error, Result};
use nom::{character::complete::multispace0, multi::many0, sequence::tuple};

use crate::app::config::Config;

use super::token::{token, Token, TokenKind};

pub struct TokenIs {
    pub value: String,
    pub kind: TokenKind,
}

impl Criteria for TokenIs {
    fn to_string(&self) -> String {
        format!("{:?}({})", self.kind, self.value)
    }
}

pub trait Criteria {
    fn to_string(&self) -> String;
}

pub struct Filter {
    pub criterias: Vec<Box<dyn Criteria>>,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let strings: Vec<String> = self
            .criterias
            .iter()
            .map(|criteria| criteria.to_string())
            .collect();

        f.write_str(&strings.join(" && "))?;
        Ok(())
    }
}

pub fn parse_filter<'a>(text: &'a str, config: &Config) -> Result<Filter> {
    let tokens = many0(tuple((|input| token(input, config), multispace0)))(text);

    match tokens {
        Ok(ok) => Ok(Filter {
            criterias: ok.1.iter().fold(vec![], |mut carry, result| {
                carry.push(Box::new(TokenIs {
                    value: result.0.clone().text,
                    kind: result.0.kind,
                }));
                carry
            }),
        }),
        Err(_) => Err(Error::msg("foo")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter() {
        let parsed = parse_filter("@foobar", &Config::empty()).unwrap();
        assert_eq!(1, parsed.criterias.len());
        assert_eq!("Tag(foobar)", parsed.criterias[0].to_string())
    }

    #[test]
    fn test_parse_filters() {
        let parsed = parse_filter("@foobar ~@barfoo", &Config::empty()).unwrap();
        assert_eq!(2, parsed.criterias.len());
        assert_eq!("And(Tag(foobar),Not(barfoo))", parsed.to_string())
    }
}
