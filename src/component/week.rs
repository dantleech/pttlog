use anyhow::{Error, Result};
use tui::{backend::Backend, layout::Rect, Frame};

use crate::{app::config::KeyMap, model::entries::LogDays};

pub struct Week {
    pub initialized: bool,
}

impl Week {
    pub fn new() -> Week {
        Week { initialized: false }
    }

    pub fn draw<B: Backend>(
        &mut self,
        _f: &mut Frame<B>,
        _are: Rect,
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
