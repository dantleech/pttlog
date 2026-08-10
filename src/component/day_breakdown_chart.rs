use anyhow::{Ok, Result};
use tui::{
    backend::Backend,
    layout::{Margin, Rect},
    style::{Color, Style},
    widgets::BarChart,
    Frame,
};

use crate::model::model::LogContext;

pub struct DayBreakdownChart {}

impl DayBreakdownChart {
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, context: &LogContext) -> Result<()> {
        f.render_widget(
            BarChart::default()
                .data(&context.log_days.minutes_by_weekday())
                .bar_style(Style::default().fg(Color::Red))
                .value_style(Style::default().fg(Color::Green))
                .bar_width(5),
            area.inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }),
        );
        Ok(())
    }
}
