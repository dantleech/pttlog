use anyhow::{Ok, Result};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::model::model::{LogDays, LogDuration};

pub struct DayBreakdownTable {}

impl DayBreakdownTable {
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, log_days: &LogDays) -> Result<()> {
        let mut rows = vec![];
        let binding = ["Day", "Hours"];
        let headers = binding
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));

        for (day, minutes) in log_days.minutes_by_weekday() {
            let duration = LogDuration::from_minutes(minutes.try_into().unwrap());
            rows.push(Row::new([
                Cell::from(day),
                Cell::from(duration.to_string()),
            ]));
        }

        let table = Table::new(rows)
            .header(
                Row::new(headers)
                    .height(1)
                    .bottom_margin(1)
                    .style(Style::default()),
            )
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
        f.render_widget(table, area);
        Ok(())
    }
}
