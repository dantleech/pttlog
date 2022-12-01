use anyhow::Error;
use tui::{backend::Backend, Frame};

use crate::app::config::Key;

pub struct Filter {}

impl Filter {
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>, filter: String) -> Result<(), Error> {
        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &Key) {}
}
