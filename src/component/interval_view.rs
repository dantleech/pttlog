use anyhow::{Error, Result};
use chrono::{Duration, Local, NaiveDate};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
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
    duration: Duration,
}

impl IntervalView<'_> {
    pub fn new<'a>(start_date: NaiveDate, duration: Duration) -> IntervalView<'a> {
        IntervalView {
            initialized: false,
            duration,
            date_start: start_date - duration,
            date_end: start_date,
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
        let log_days = log_days.between(self.date_start, self.date_end);

        let container = Block::default().borders(Borders::ALL).title(format!(
            "{} from {} until {}",
            self.duration.to_string(),
            self.date_start.format("%A %e %B"),
            self.date_end.format("%A %e %B, %Y"),
        ));

        f.render_widget(
            container,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        );

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(area.inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        self.tag_summary
            .draw(f, summary_rows[0], &log_days.tag_summary(TokenKind::Tag))?;
        self.ticket_summary
            .draw(f, summary_rows[1], &log_days.tag_summary(TokenKind::Ticket))?;

        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &KeyMap) {
        match key {
            KeyMap::PreviousPage => {
                self.date_start -= self.duration;
                self.date_end -= self.duration;
            }
            KeyMap::NextPage => {
                self.date_start += self.duration;
                self.date_end += self.duration;
            }
            _ => (),
        };
    }
}
