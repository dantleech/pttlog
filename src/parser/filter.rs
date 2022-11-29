use anyhow::{Error, Result};
use nom::{character::complete::multispace1, multi::many0, sequence::tuple, IResult};

use crate::app::config::Config;

use super::token::{token, Token};

pub trait Criteria {}

#[derive(Debug)]
pub struct Filter {
    pub tokens: Vec<Token>,
}

pub fn parse_filter<'a>(text: &'a str, config: &Config) -> Result<Filter> {
    let tokens = many0(tuple((|input| token(input, config), multispace1)))(text);

    match tokens {
        Ok(ok) => Ok(Filter {
            tokens: ok.1.iter().fold(vec![], |mut carry, result| {
                carry.push(result.0.clone());
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
        assert_eq!(1, parsed.tokens.len());
        assert_eq!("foo", parsed.tokens[0].to_string())
    }
}
