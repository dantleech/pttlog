use anyhow::{Error, Result};
use chrono::{Duration, Local, NaiveDate};
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{app::config::KeyMap, model::entries::LogDays};

pub struct IntervalView {
    initialized: bool,
    date_start: NaiveDate,
    date_end: NaiveDate,
}

impl IntervalView {
    pub fn new() -> IntervalView {
        IntervalView {
            initialized: false,
            date_start: Local::now().date_naive() - Duration::days(7),
            date_end: Local::now().date_naive(),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        _f: &mut Frame<B>,
        _area: Rect,
        _log_days: &LogDays,
    ) -> Result<(), Error> {
        // default to lastest entry
        if !self.initialized {
            self.initialized = true;
        }

        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &KeyMap) {
        match key {
            _ => (),
        };
    }
}
