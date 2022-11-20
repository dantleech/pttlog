use anyhow::{Error, Result};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{model::entries::LogDays, parser::TokenKind};

use super::{log_table::LogTable, token_summary_table::TokenSummaryTable, Component};

pub struct Day<'a> {
    pub index: usize,
    pub log_table: LogTable,
    pub tag_summary: TokenSummaryTable<'a>,
    pub ticket_summary: TokenSummaryTable<'a>,
}

impl Day<'_> {
    pub fn new<'a>() -> Day<'a> {
        Day {
            index: 0,
            log_table: LogTable {},
            tag_summary: TokenSummaryTable::new("Tags", TokenKind::Tag),
            ticket_summary: TokenSummaryTable::new("Tickets", TokenKind::Ticket),
        }
    }

    pub fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>,
        area: Rect,
        log_days: &LogDays,
    ) -> Result<(), Error> {
        let log_day = log_days.at(self.index);
        let columns = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area.inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        let container = Block::default()
            .borders(Borders::ALL)
            .title(log_day.date().to_verbose_string());

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[1]);

        self.log_table.draw(f, columns[0], &log_day)?;

        self.tag_summary.draw(f, summary_rows[0], &log_day)?;
        self.ticket_summary.draw(f, summary_rows[1], &log_day)?;

        f.render_widget(
            container,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        );

        Ok(())
    }
}

impl Component for Day<'_> {}
