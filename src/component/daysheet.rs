use anyhow::{Error, Result};
use tui::{
    backend::Backend, layout::{Constraint, Layout, Margin, Rect}, style::{Color, Style}, text::Span, widgets::{Block, Borders, Cell, Row, Table}, Frame
};

use crate::{app::config::KeyName, model::model::LogDays, parser::token::TokenKind};

use super::{log_table::LogTable, token_summary_table::TokenSummaryTable};

pub struct Daysheet {
}

impl Daysheet {
    pub fn new() -> Daysheet {
        Daysheet {
        }
    }

    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        area: Rect,
        log_days: &LogDays,
    ) -> Result<(), Error> {
        let headers = ["Date", "Duration"]
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));
        let rows: Vec<Row> = log_days.entries.iter().map(|e| {
            Row::new(vec![
                e.date().to_verbose_string(),
                e.duration_total().to_string(),
            ])
        }).collect();

        f.render_widget(
            Table::new(rows)
                .header(
                    Row::new(headers)
                        .height(1)
                        .bottom_margin(1)
                        .style(Style::default()),
                )
                .widths(&[
                    Constraint::Percentage(65),
                    Constraint::Percentage(65),
                ]),
            area
        );

        Ok(())
    }

    fn next(&mut self) {
    }

    fn previous(&mut self) {
    }

    pub(crate) fn handle(&mut self, key: &KeyName) {
        match key {
            KeyName::PreviousPage => self.previous(),
            KeyName::NextPage => self.next(),
            _ => (),
        };
    }
}
