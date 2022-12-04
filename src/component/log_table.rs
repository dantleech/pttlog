use anyhow::Ok;
use chrono::{Local, Timelike};
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table},
};

use crate::{
    model::model::{LogDay, TimeRangeView},
    parser::{
        timesheet::Tokens,
        token::{Token, TokenKind},
    },
};

pub struct LogTable {}

impl LogTable {
    pub fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        log_day: &LogDay,
    ) -> anyhow::Result<()> {
        let mut rows = vec![];
        let headers = ["Time", "Duration", "Description", ""]
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));
        let _duration_total = log_day.duration_total();
        let now = Local::now().naive_local();

        for log in log_day.iter() {
            rows.push(Row::new([
                Cell::from((|time_range: &TimeRangeView| {
                    // 1. if today and end time not set show "now"
                    Spans::from(vec![
                        Span::raw(time_range.start.format("%H:%M").to_string()),
                        Span::styled("-", Style::default().fg(Color::DarkGray)),
                        (|| {
                            if time_range.ongoing {
                                return Span::styled(
                                    now.format("%H:%M").to_string(),
                                    Style::default()
                                        .fg((|| {
                                            if 0 == now.num_seconds_from_midnight() % 2 {
                                                Color::Black
                                            } else {
                                                Color::Gray
                                            }
                                        })())
                                        .bg(Color::DarkGray),
                                );
                            }
                            Span::styled(
                                time_range.end.format("%H:%M").to_string(),
                                Style::default().fg(Color::DarkGray),
                            )
                        })(),
                        (|| Span::raw(""))(),
                    ])
                })(&log.time_range())),
                Cell::from((|range: &TimeRangeView| -> Spans {
                    Spans::from(vec![
                        Span::raw(range.duration().to_string()),
                        Span::styled(
                            format!(
                                " {:.2}%",
                                log.percentage_of_day(log_day.duration_total().num_minutes())
                            ),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ])
                })(&log.time_range())),
                Cell::from(description(&log.description())),
            ]));
        }

        rows.push(Row::new([
            Cell::default(),
            Cell::default(),
            Cell::default(),
        ]));
        rows.push(Row::new([
            Cell::from(Span::styled("Total:", Style::default().fg(Color::DarkGray))),
            Cell::from(Span::raw(log_day.duration_total().to_string())),
            Cell::default(),
        ]));

        f.render_widget(
            Table::new(rows)
                .header(
                    Row::new(headers)
                        .height(1)
                        .bottom_margin(1)
                        .style(Style::default()),
                )
                .widths(&[
                    Constraint::Length(11),
                    Constraint::Length(11),
                    Constraint::Percentage(65),
                ]),
            area,
        );
        Ok(())
    }
}

fn description(tokens: &Tokens) -> Spans {
    let foo = tokens
        .to_vec()
        .iter()
        .map(|t: &Token| match t.kind {
            TokenKind::Tag => Span::styled(
                format!("@{}", t.to_string().to_owned()),
                Style::default().fg(Color::Green),
            ),
            TokenKind::Prose => Span::raw(t.to_string().to_owned()),
            TokenKind::Ticket => Span::styled(
                format!("{}", t.to_string().to_owned()),
                Style::default().fg(Color::Cyan),
            ),
        })
        .collect::<Vec<_>>();
    Spans::from(foo)
}
