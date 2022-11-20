use anyhow::{Error, Result};
use chrono::{Duration, NaiveDate};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{app::config::KeyMap, model::entries::LogDays, parser::TokenKind};

use super::{day_breakdown_chart::DayBreakdownChart, token_summary_table::TokenSummaryTable};

pub struct IntervalView<'a> {
    initialized: bool,
    date_start: NaiveDate,
    date_end: NaiveDate,
    tag_summary: TokenSummaryTable<'a>,
    ticket_summary: TokenSummaryTable<'a>,
    duration: Duration,
    day_breakdown_chart: DayBreakdownChart,
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
            day_breakdown_chart: DayBreakdownChart {},
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
            "{} from {} {} until {}",
            self.duration.to_string(),
            self.date_start.format("%A"),
            self.date_start.to_string(),
            self.date_end.to_string()
        ));

        f.render_widget(
            container,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        );

        let columns = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area.inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[1].inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        self.day_breakdown_chart.draw(f, area, &log_days)?;
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
