use nom::branch;
use nom::bytes::complete::{self, tag};
use nom::character::complete::char;
use nom::character::complete::{alphanumeric1, space0};
use nom::error::{Error, ErrorKind};
use nom::sequence::tuple;

use crate::app::config::Config;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Prose,
    Tag,
    Ticket,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub whitespace: String,
}

impl Token {
    pub fn tag(text: String) -> Token {
        Token {
            kind: TokenKind::Tag,
            text,
            whitespace: "".to_string(),
        }
    }
    pub fn prose(text: String) -> Token {
        Token {
            kind: TokenKind::Prose,
            text,
            whitespace: "".to_string(),
        }
    }
    pub fn ticket(identifier: String) -> Token {
        Token {
            kind: TokenKind::Ticket,
            text: identifier,
            whitespace: "".to_string(),
        }
    }
    pub fn to_string(&self) -> String {
        format!("{}{}", self.text, self.whitespace)
    }
    pub fn text(&self) -> &str {
        &self.text
    }
}

pub fn tag_token(text: &str) -> nom::IResult<&str, Token> {
    let token = tuple((char('@'), alphanumeric1, space0))(text);

    match token {
        Ok(ok) => Ok((
            ok.0,
            Token {
                kind: TokenKind::Tag,
                text: (ok.1).1.to_string(),
                whitespace: (ok.1).2.to_string(),
            },
        )),
        Err(err) => Err(err),
    }
}
pub fn ticket_token<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Token> {
    for project in config.projects.iter() {
        let input = text;
        match tuple((
            tag::<_, _, Error<&str>>(project.ticket_prefix.as_str()),
            alphanumeric1,
            space0,
        ))(input)
        {
            Ok(ok) => {
                return Ok((
                    ok.0,
                    Token {
                        kind: TokenKind::Ticket,
                        text: format!("{}{}", (ok.1).0, (ok.1).1),
                        whitespace: (ok.1).2.to_string(),
                    },
                ))
            }
            Err(_err) => (),
        }
    }

    Err(nom::Err::Error(Error::new(text, ErrorKind::Tag)))
}

fn prose_token(text: &str) -> nom::IResult<&str, Token> {
    let text = tuple((
        space0,
        complete::take_till1(|c| c == ' ' || c == '\n' || c == '\r'),
        space0,
    ))(text);

    match text {
        Ok(ok) => {
            let spaces1 = (ok.1).0;
            let word = (ok.1).1.to_string();
            let spaces2 = (ok.1).2;

            Ok((
                ok.0,
                Token {
                    kind: TokenKind::Prose,
                    text: format!("{}{}", spaces1, word),
                    whitespace: spaces2.to_string(),
                },
            ))
        }
        Err(err) => Err(err),
    }
}

pub fn token<'a>(text: &'a str, config: &Config) -> nom::IResult<&'a str, Token> {
    branch::alt((tag_token, |input| ticket_token(input, config), prose_token))(text)
}
