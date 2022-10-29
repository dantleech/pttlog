use super::app;

use super::parser;
use tui::{widgets::{Table, Block, Row, Borders, Cell, Paragraph}, layout::{Constraint, Layout}, style::Style, Frame, backend::Backend, text::{Span, Spans}};

pub fn layout<B: Backend>(f: &mut Frame<B>, app: &app::App) {
    let rows = Layout::default()
        .margin(0)
        .constraints([
                     Constraint::Min(2),
                     Constraint::Percentage(100)
            ].as_ref()
        ).split(f.size());

    f.render_widget(navigation(app), rows[0]);
    f.render_widget(table(app.current_entry()), rows[1])
}

fn navigation(app: &app::App) -> Paragraph {
    let text: Vec<Spans> = vec![
        Spans::from(vec![
           Span::raw("PTTLog "),
           Span::raw(format!("{}/{} ", app.current_entry_number().to_string(), app.entry_count().to_string())),
           Span::raw("[p]rev "),
           Span::raw(app.current_entry().date.to_string()),
           Span::raw(" [n]ext")
        ]),
        Spans::from(vec![
            Span::raw(app.current_entry().duration_total_as_string())
        ])
    ];

    Paragraph::new(text)
}

pub fn table(entry: &parser::Entry) -> Table {
    let mut rows = vec![];
    let headers = ["Start", "Duration", "Description"]
        .iter()
        .map(|header| Cell::from(*header));

        for log in entry.logs.iter() {
            rows.push(Row::new([
                Cell::from(log.time.to_string()),
                Cell::from(log.duration_as_string()),
                Cell::from(log.description.as_str()),
            ]));
        }

    Table::new(rows)
        .header(Row::new(headers).height(1).bottom_margin(1).style(Style::default()))
        .widths(&[
            Constraint::Min(7),
            Constraint::Min(8),
            Constraint::Percentage(33),
        ])
        .block(Block::default().borders(Borders::ALL).title("Timesheet"))
}
