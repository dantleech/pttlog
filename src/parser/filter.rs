use nom::IResult;

use crate::app::config::Config;

pub trait Criteria {}
pub struct Filter {
    pub tokens: Vec<Box<dyn Criteria>>,
}

pub fn parse_filter<'a>(text: &'a str, config: &Config) -> IResult<&'a str, Filter> {
    Ok(("", Filter { tokens: vec![] }))
}
