use std::fmt::Display;

use anyhow::{Error, Result};
use chrono::{Duration, Months, NaiveDate};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    app::config::KeyName,
    model::{model::LogDays, time::TimeFactory},
    parser::token::TokenKind,
};

use super::{
    day_breakdown_chart::DayBreakdownChart, day_breakdown_table::DayBreakdownTable,
    token_summary_table::TokenSummaryTable,
};

pub struct IntervalView<'a> {
    initialized: bool,
    date_start: NaiveDate,
    date_end: NaiveDate,
    time: &'a dyn TimeFactory,
    tag_summary: TokenSummaryTable<'a>,
    ticket_summary: TokenSummaryTable<'a>,
    duration: ReportDuration,
    day_breakdown_chart: DayBreakdownChart,
    day_breakdown_table: DayBreakdownTable,
}

#[derive(Clone, Copy)]
pub enum ReportDuration {
    Day,
    Week,
    Month,
    Year,
}

impl Display for ReportDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReportDuration::Day => "day",
            ReportDuration::Week => "week",
            ReportDuration::Month => "month",
            ReportDuration::Year => "year",
        })
    }
}

impl IntervalView<'_> {
    pub fn new<'a>(
        time: &'a dyn TimeFactory,
        start_date: NaiveDate,
        duration: ReportDuration,
    ) -> IntervalView<'a> {
        IntervalView {
            initialized: false,
            duration,
            date_start: start_date,
            date_end: shift_range(&duration, start_date, 1),
            time,
            tag_summary: TokenSummaryTable::new("Tags"),
            ticket_summary: TokenSummaryTable::new("Tickets"),
            day_breakdown_chart: DayBreakdownChart {},
            day_breakdown_table: DayBreakdownTable {},
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
        let log_days = log_days.until(self.date_start, self.date_end);

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

        let left_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[0].inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));
        self.day_breakdown_chart.draw(f, left_rows[0], &log_days)?;
        self.day_breakdown_table.draw(f, left_rows[1], &log_days)?;

        let right_rows = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Min(2)])
            .split(columns[1].inner(&Margin {
                vertical: 2,
                horizontal: 2,
            }));

        self.tag_summary
            .draw(f, right_rows[0], &log_days.tag_summary(TokenKind::Tag))?;
        self.ticket_summary
            .draw(f, right_rows[1], &log_days.tag_summary(TokenKind::Ticket))?;

        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &KeyName) {
        match key {
            KeyName::PreviousPage => {
                self.date_start = shift_range(&self.duration, self.date_start, -1);
                self.date_end = shift_range(&self.duration, self.date_end, -1);
            }
            KeyName::NextPage => {
                self.date_start = shift_range(&self.duration, self.date_start, 1);
                self.date_end = shift_range(&self.duration, self.date_end, 1);
            }
            _ => (),
        };
    }
}
fn shift_range(duratinon: &ReportDuration, date: NaiveDate, amount: i64) -> NaiveDate {
    match duratinon {
        ReportDuration::Day => date + Duration::days(amount),
        ReportDuration::Week => date + Duration::weeks(amount),
        ReportDuration::Month => match (amount < 0) {
            true => date.checked_sub_months(Months::new((-amount).try_into().unwrap())).unwrap(),
            false => date.checked_add_months(Months::new(amount.try_into().unwrap())).unwrap(),
        }
        ReportDuration::Year => date + Duration::days(amount * 365),
    }

}


#[cfg(test)]
mod test {
    use crate::model::time::FrozenTimeFactory;

    use super::*;

    #[test]
    fn next_and_previous_pages() {
        let time = FrozenTimeFactory::new(2023, 1, 1, 12, 1);
        let mut view = IntervalView::new(
            &time,
            FrozenTimeFactory::new(2022, 1, 1, 12, 1).now().date(),
            Duration::weeks(1),
        );

        // 1 week forwards
        view.handle(&KeyName::NextPage);
        assert_eq!("2022-01-08", view.date_start.to_string());

        // 1 week back
        view.handle(&KeyName::PreviousPage);
        assert_eq!("2022-01-01", view.date_start.to_string());
    }

    #[test]
    fn cannot_advance_further_than_current_time() {
        let time = FrozenTimeFactory::new(2022, 1, 1, 12, 1);
        let mut view = IntervalView::new(
            &time,
            FrozenTimeFactory::new(2022, 1, 1, 12, 1).now().date(),
            Duration::weeks(1),
        );

        // 1 week forwards (does not advance)
        view.handle(&KeyName::NextPage);
        assert_eq!("2022-01-01", view.date_start.to_string());

        // 1 week back
        view.handle(&KeyName::PreviousPage);
        assert_eq!("2021-12-25", view.date_start.to_string());
    }
}
