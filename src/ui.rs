

use crate::parser::TimeRange;

use super::app;

use super::parser;
use tui::layout::Alignment;
use tui::layout::Margin;
use tui::style::Color;

use tui::widgets::Block;
use tui::widgets::Borders;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::Style,
    text::{Span, Spans},
    widgets::{Cell, Paragraph, Row, Table},
    Frame,
};

pub fn layout<B: Backend>(f: &mut Frame<B>, app: &app::App) {
    let rows = Layout::default()
        .margin(0)
        .constraints([Constraint::Min(2), Constraint::Percentage(100)].as_ref())
        .split(f.size());

    f.render_widget(navigation(app), rows[0]);

    let current_entry = app.current_entry();
    // By day view
    let columns = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(100)])
        .split(rows[1].inner(&Margin {
            vertical: 2,
            horizontal: 2,
        }));

    let container = Block::default().borders(Borders::ALL).title(
        current_entry
            .date_object()
            .format("%A %e %B, %Y")
            .to_string(),
    );
    f.render_widget(table(current_entry), columns[0]);
    f.render_widget(
        container,
        rows[1].inner(&Margin {
            vertical: 0,
            horizontal: 0,
        }),
    );
    if app.notification.should_display() {
        let text: Vec<Spans> = vec![Spans::from(vec![Span::raw(&app.notification.notification)])];

        let notification = Paragraph::new(text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(
            notification,
            rows[0].inner(&Margin {
                vertical: 0,
                horizontal: 0,
            }),
        )
    }
}

fn navigation(app: &app::App) -> Paragraph {
    let text: Vec<Spans> = vec![Spans::from(vec![
        Span::raw(format!(
            "{}/{} ",
            app.current_entry_number().to_string(),
            app.entry_count().to_string()
        )),
        Span::styled("[p]", Style::default().fg(Color::Green)),
        Span::raw("rev "),
        Span::styled("[n]", Style::default().fg(Color::Green)),
        Span::raw("ext "),
        Span::styled("[r]", Style::default().fg(Color::Green)),
        Span::raw("eload"),
        Span::styled(" [q]", Style::default().fg(Color::Green)),
        Span::raw("uit"),
    ])];

    Paragraph::new(text)
}

pub fn table(entry: &parser::Entry) -> Table {
    let mut rows = vec![];
    let headers = ["Time", "Duration", "Description"]
        .iter()
        .map(|header| Cell::from(*header));
    let entry_duration = entry.duration_total();

    for log in entry.logs.iter() {
        rows.push(Row::new([
            Cell::from((|time: &TimeRange| {
                if time.end.is_none() {
                    return Spans::from(vec![Span::raw(time.start.to_string())]);
                }
                Spans::from(vec![
                    Span::raw(time.start.to_string()),
                    Span::styled("-", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        time.end.unwrap().to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })(&log.time)),
            Cell::from(Spans::from(vec![
                Span::raw(log.duration_as_string()),
                Span::styled(
                    format!(" {:.2}%", log.as_percentage(entry_duration)),
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
            Cell::from(description(&log.description)),
        ]));
    }

    rows.push(Row::new([
       Cell::default(),
       Cell::default(),
       Cell::default(),
    ]));
    rows.push(Row::new([
       Cell::from(Span::raw("Total:")),
       Cell::from(Span::raw(entry.duration_total_as_string())),
       Cell::default(),
    ]));

    Table::new(rows)
        .header(
            Row::new(headers)
                .height(1)
                .bottom_margin(1)
                .style(Style::default()),
        )
        .widths(&[
            Constraint::Min(12),
            Constraint::Min(14),
            Constraint::Percentage(100),
        ])
}

fn description(tokens: &parser::Tokens) -> Spans<'static> {
    let foo = tokens
        .to_vec()
        .iter()
        .map(|t: &parser::Token| match t.kind {
            parser::TokenKind::Tag => Span::styled(
                format!("@{}", t.text.to_owned()),
                Style::default().fg(Color::Green),
            ),
            parser::TokenKind::Prose => Span::raw(t.text.to_owned()),
        })
        .collect::<Vec<_>>();
    Spans::from(foo)
}
