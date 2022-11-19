use anyhow::{Error, Result};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{model::entries::LogEntries, parser::TokenKind};

use super::{log_table::LogTable, token_summary_table::TokenSummaryTable, Component};

pub struct Day<'a> {
    pub entries: LogEntries<'a>,
    pub index: usize,
    pub log_table: LogTable<'a>,
    pub tag_summary: TokenSummaryTable<'a>,
    pub ticket_summary: TokenSummaryTable<'a>,
}

impl Day<'_> {
    pub fn new(entries: LogEntries) -> Day {
        Day {
            entries,
            index: 0,
            log_table: LogTable { entries },
            tag_summary: TokenSummaryTable::new("Tags", TokenKind::Tag, entries),
            ticket_summary: TokenSummaryTable::new("Tickets", TokenKind::Ticket, entries),
        }
    }
}

impl Component for Day<'_> {
    fn next(&mut self) {
        if self.index == self.entries.len() - 1 {
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

    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) -> Result<(), Error> {
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
            .title(self.entries.date().to_verbose_string());

        let summary_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[1]);

        self.log_table.draw(f, columns[0]);

        self.tag_summary.draw(f, summary_rows[0]);
        self.ticket_summary.draw(f, summary_rows[1]);

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
