use anyhow::{Error, Result};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{app::config::KeyMap, model::entries::LogDays, parser::TokenKind};

use super::{log_table::LogTable, token_summary_table::TokenSummaryTable};

pub struct Day<'a> {
    pub index: usize,
    pub log_table: LogTable,
    pub tag_summary: TokenSummaryTable<'a>,
    pub ticket_summary: TokenSummaryTable<'a>,
    pub initialized: bool,
}

impl Day<'_> {
    pub fn new<'a>() -> Day<'a> {
        Day {
            index: 0,
            log_table: LogTable {},
            tag_summary: TokenSummaryTable::new("Tags"),
            ticket_summary: TokenSummaryTable::new("Tickets"),
            initialized: false,
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
            self.index = log_days.len();
            self.initialized = true;
        }
        // do not allow overflow
        if self.index >= log_days.len() {
            self.index = log_days.len() - 1
        }

        let log_day = log_days.at(self.index);

        let columns = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area.inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        let container = Block::default().borders(Borders::ALL).title(format!(
            "{}/{} {}",
            self.index + 1,
            log_days.len(),
            log_day.date().to_verbose_string()
        ));

        self.log_table.draw(f, columns[0], &log_day)?;

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[1]);

        self.tag_summary
            .draw(f, summary_rows[0], &log_day.tag_summary(TokenKind::Tag))?;
        self.ticket_summary
            .draw(f, summary_rows[1], &log_day.tag_summary(TokenKind::Ticket))?;

        f.render_widget(
            container,
            area.inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        );

        Ok(())
    }

    fn next(&mut self) {
        self.index += 1
    }

    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1
        }
    }

    pub(crate) fn handle(&mut self, key: &KeyMap) {
        match key {
            KeyMap::PreviousPage => self.previous(),
            KeyMap::NextPage => self.next(),
            _ => (),
        };
    }
}
