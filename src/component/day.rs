use anyhow::{Result, Error};
use tui::{backend::Backend, Frame, layout::Rect};

use crate::parser::Entries;

use super::Component;

pub struct Day {
    pub entries: Entries,
    pub index: usize,
}

impl Component for Day {
    fn next(&mut self) {
        if self.index == self.entries.entries.len() - 1 {
            return;
        }
        self.index += 1;
    }

    fn prev(&mut self) {
        if self.index == 0 {
            return;
        }
        self.index -= 1;
    }

    fn draw<B: Backend>(
		&self,
		f: &mut Frame<B>,
		rect: Rect,
	) -> Result<(), Error> {
        todo!()
    }
}


