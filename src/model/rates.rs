
use crate::{app::config::Config, model::model::{LogDuration, Money}};
use iso_currency::{Currency};

#[derive(Clone)]
pub struct Rate {
    pub ticket_prefix: Option<String>,
    pub tags: Vec<String>,
    pub rate: u64,
    pub currency: Currency,
}

impl Rate {
    pub(crate) fn cost(&self, duration: &LogDuration) -> Money {
        Money::new(
            self.currency,
            ((duration.num_minutes() as f64 / 60.0) * self.rate as f64).round() as u64
        )
    }
}

#[derive(Clone, Default)]
pub struct Rates {
    rates: Vec<Rate>
}

impl Rates {
    #[allow(dead_code)]
    pub(crate) fn from_rates(rates: Vec<Rate>) -> Rates {
        Rates{rates}
    }
    pub(crate) fn from_config(config: &Config) -> Rates {
        let mut rates = vec![];
        for project in &config.projects {
            if let Some(rate) = &project.rate {
                rates.push(Rate{
                    ticket_prefix: Some(project.ticket_prefix.clone()),
                    tags: project.tags.clone(),
                    rate: rate.rate,
                    currency: rate.currency
                });
            }
        }
        Rates { rates }
    }

    pub(crate) fn for_tag(&self, tag: &String) -> Vec<Rate> {
        for rate in &self.rates {
            if !rate.tags.contains(tag) {
                continue;
            }

            return vec![rate.clone()];
        }

        vec![]
    }

    pub(crate) fn for_ticket(&self, ticket: &String) -> Vec<Rate> {
        for rate in &self.rates {
            let prefix = match &rate.ticket_prefix {
                Some(prefix) => prefix,
                None => continue,
            };

            if !ticket.starts_with(prefix) {
                continue;
            }

            return vec![rate.clone()];
        }

        vec![]
    }
}

#[cfg(test)]
mod test {
    use crate::app::config::Project;
    use crate::app::config::Rate as ConfigRate;

use super::*;

    #[test]
    pub fn test_rates_from_config() {
        let mut config = Config::empty();
        config.projects = vec![
            Project{
                name: "Hello".to_string(),
                ticket_prefix: "HELLO-".to_string(),
                tags: vec!["one".to_string(), "two".to_string()],
                rate: Some(
                    ConfigRate{
                        rate: 100,
                        currency:iso_currency::Currency::USD 
                    }
                ),
            }
        ];
        let rates = Rates::from_config(&config);
        let rates = rates.for_tag(&"one".to_string());

        assert_eq!(100, rates[0].rate);
    }

    #[test]
    fn test_rate() {
        let rate = Rate{
            ticket_prefix: Some("FOOBAR-".to_string()),
            tags: vec!["foobar".to_string()],
            rate: 100,
            currency: Currency::AED
        };

        assert_eq!(
            500,
            rate.cost(&LogDuration::from_minutes(60 * 5)).amount
        );
        assert_eq!(
            108,
            rate.cost(&LogDuration::from_minutes(65)).amount
        );
    }
}
