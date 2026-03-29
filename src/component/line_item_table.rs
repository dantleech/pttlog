use anyhow::Ok;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Cell, Row, Table},
};

use crate::{
    model::model::LogDays,
    parser::{
        timesheet::Tokens,
        token::{Token, TokenKind},
    },
};

pub struct LineItemTable {}

impl LineItemTable {
    pub fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        days: &LogDays,
    ) -> anyhow::Result<()> {
        let mut rows = vec![];
        let headers = ["Date", "Description", "Duration"]
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));
        let _duration_total = days.duration_total();

        for day in days.iter() {
            rows.push(Row::new([
                Cell::from(
                    Spans::from(vec![
                        Span::raw(day.date().to_compact_string()),
                        Span::styled("-", Style::default().fg(Color::DarkGray)),
                    ])
                ),
                Cell::from(description_non_ref(day.description().clone())),
                Cell::from(
                    Spans::from(vec![
                        Span::raw(day.duration_total().to_string()),
                    ])
                ),
            ]));
        }
        rows.push(Row::new([
            Cell::default(),
            Cell::default(),
            Cell::default(),
        ]));
        rows.push(Row::new([
            Cell::from(Span::styled("Total:", Style::default().fg(Color::DarkGray))),
            Cell::default(),
            Cell::from(Span::raw(days.duration_total().to_string())),
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
                    Constraint::Percentage(65),
                    Constraint::Length(11),
                ]),
            area,
        );
        Ok(())
    }
}

fn description_non_ref<'a>(tokens: Tokens) -> Spans<'a> {
    let foo: Vec<Span> = tokens
        .to_vec()
        .iter()
        .map(|t: &Token| match t.kind {
            TokenKind::Tag => Span::styled(
                format!("@{}", t.to_string().to_owned()),
                Style::default().fg(Color::Green),
            ),
            TokenKind::Prose => Span::raw(t.to_string().to_owned()),
            TokenKind::Ticket => Span::styled(
                t.to_string().to_owned().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        })
        .collect();
    Spans::from(foo)
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
                t.to_string().to_owned().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        })
        .collect::<Vec<_>>();
    Spans::from(foo)
}

