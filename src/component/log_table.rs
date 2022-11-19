use anyhow::Ok;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table},
};

use crate::{
    model::entries::{LogDay, TimeRangeView},
    parser,
};

use super::Component;

pub struct LogTable<'a> {
    entries: LogDay<'a>,
}

impl Component for LogTable<'_> {
    fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
    ) -> anyhow::Result<()> {
        let mut rows = vec![];
        let headers = ["Time", "Duration", "Description", ""]
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));
        let _duration_total = self.entries.duration_total();

        for log in self.entries.iter() {
            rows.push(Row::new([
                Cell::from((|time_range: &TimeRangeView| {
                    // 1. if today and end time not set show "now"
                    Spans::from(vec![
                        Span::raw(time_range.start.format("%H:%M").to_string()),
                        Span::styled("-", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            if time_range.ongoing {
                                "now".to_string()
                            } else {
                                time_range.end.format("%H:%M").to_string()
                            },
                            Style::default().fg(Color::DarkGray),
                        ),
                        (|| Span::raw(""))(),
                    ])
                })(&log.time_range())),
                Cell::from((|range: &TimeRangeView| -> Spans {
                    Spans::from(vec![
                        Span::raw(range.duration().to_string()),
                        Span::styled(
                            format!(
                                " {:.2}%",
                                log.percentage_of_day(self.entries.duration_total().num_minutes())
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
            Cell::from(Span::raw(self.entries.duration_total().to_string())),
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

fn description(tokens: &parser::Tokens) -> Spans {
    let foo = tokens
        .to_vec()
        .iter()
        .map(|t: &parser::Token| match t.kind {
            parser::TokenKind::Tag => Span::styled(
                format!("@{}", t.to_string().to_owned()),
                Style::default().fg(Color::Green),
            ),
            parser::TokenKind::Prose => Span::raw(t.to_string().to_owned()),
            parser::TokenKind::Ticket => Span::styled(
                format!("{}", t.to_string().to_owned()),
                Style::default().fg(Color::Cyan),
            ),
        })
        .collect::<Vec<_>>();
    Spans::from(foo)
}