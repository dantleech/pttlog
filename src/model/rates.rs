
use crate::{app::config::Config, model::model::{LogDuration, Money}};
use chrono::NaiveDate;
use iso_currency::{Currency};

#[derive(Clone)]
pub struct Epoch {
    pub ticket_prefix: Option<String>,
    pub tags: Vec<String>,
    pub rate: u64,
    pub currency: Currency,
    pub from: NaiveDate,
}

impl Epoch {
    pub(crate) fn cost_for_duration(&self, duration: &LogDuration) -> Money {
        Money::new(
            self.currency,
            ((duration.num_minutes() as f64 / 60.0) * self.rate as f64).round() as u64
        )
    }
}

#[derive(Clone, Default)]
pub struct Epochs {
    epochs: Vec<Epoch>
}

impl Epochs {
    #[allow(dead_code)]
    pub(crate) fn from_rates(epochs: Vec<Epoch>) -> Epochs {
        Epochs{epochs}
    }
    pub(crate) fn from_config(config: &Config) -> Epochs {
        let mut epochs = vec![];
        for project in &config.projects {
            for epoch in &project.epochs {
                match &epoch.rate {
                    None => continue,
                    Some(r) => {
                        epochs.push(Epoch{
                            ticket_prefix: Some(project.ticket_prefix.clone()),
                            tags: project.tags.clone(),
                            rate: r.rate,
                            currency: r.currency,
                            from: NaiveDate::from_ymd_opt(
                                epoch.from.year as i32,
                                epoch.from.month as u32,
                                epoch.from.day as u32
                            ).unwrap(),
                        });
                    },
                }
            }
        }
        Epochs { epochs }
    }

    pub(crate) fn for_tag(&self, tag: &String) -> Vec<Epoch> {
        for rate in &self.epochs {
            if !rate.tags.contains(tag) {
                continue;
            }

            return vec![rate.clone()];
        }

        vec![]
    }

    pub(crate) fn for_ticket(&self, ticket: &String) -> Vec<Epoch> {
        for rate in &self.epochs {
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

    pub(crate) fn for_days(&self, _: &NaiveDate, end_date: &NaiveDate) -> Epochs {
        let mut epochs = vec![];
        for epoch in &self.epochs {
            if &epoch.from > end_date {
                continue
            }
            epochs.push(epoch.clone())
        }
        Epochs{epochs}
    }

}

#[cfg(test)]
mod test {

use crate::app::config::Epoch as ConfigEpoch;
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
                rate: None,
                epochs: vec![
                    ConfigEpoch{
                        from: toml::value::Date{year: 2026, month: 1, day: 1},
                        rate: Some(
                            ConfigRate{
                                rate: 100,
                                currency:iso_currency::Currency::USD 
                            }
                        )
                    }
                ]
            }
        ];
        let rates = Epochs::from_config(&config);
        let rates = rates.for_tag(&"one".to_string());

        assert_eq!(100, rates[0].rate);
    }

    #[test]
    fn test_epoch() {
        let epoch = Epoch{
            from: NaiveDate::from_ymd_opt(2016, 01, 01).unwrap(),
            ticket_prefix: Some("FOOBAR-".to_string()),
            tags: vec!["foobar".to_string()],
            rate: 100,
            currency: Currency::AED
        };

        assert_eq!(
            500,
            epoch.cost_for_duration(&LogDuration::from_minutes(60 * 5)).amount
        );
        assert_eq!(
            108,
            epoch.cost_for_duration(&LogDuration::from_minutes(65)).amount
        );
    }
}
