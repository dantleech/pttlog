use anyhow::{Error, Result};
use chrono::{Duration, Local, NaiveDate};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    Frame,
};

use crate::{app::config::KeyMap, model::entries::LogDays, parser::TokenKind};

use super::token_summary_table::TokenSummaryTable;

pub struct IntervalView<'a> {
    initialized: bool,
    date_start: NaiveDate,
    date_end: NaiveDate,
    tag_summary: TokenSummaryTable<'a>,
    ticket_summary: TokenSummaryTable<'a>,
}

impl IntervalView<'_> {
    pub fn new<'a>() -> IntervalView<'a> {
        IntervalView {
            initialized: false,
            date_start: Local::now().date_naive() - Duration::days(7),
            date_end: Local::now().date_naive(),
            tag_summary: TokenSummaryTable::new("Tags", TokenKind::Tag),
            ticket_summary: TokenSummaryTable::new("Tickets", TokenKind::Ticket),
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Rect,
        log_days: &LogDays,
    ) -> Result<(), Error> {
        // default to lastest entry
        if !self.initialized {
            self.initialized = true;
        }

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(area);

        self.tag_summary
            .draw(f, summary_rows[0], &log_days.tag_summary(TokenKind::Tag))?;
        self.ticket_summary
            .draw(f, summary_rows[1], &log_days.tag_summary(TokenKind::Ticket))?;

        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &KeyMap) {
        match key {
            _ => (),
        };
    }
}
