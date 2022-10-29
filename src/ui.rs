use super::app;

use super::parser;
use tui::{widgets::{Table, Block, Row, Borders, Cell, Paragraph}, layout::{Constraint, Layout}, style::Style, Frame, backend::Backend, text::{Span, Spans}};

pub fn layout<B: Backend>(f: &mut Frame<B>, app: &app::App) {
    let rows = Layout::default()
        .margin(0)
        .constraints([
                     Constraint::Min(2),
                     Constraint::Min(2),
                     Constraint::Percentage(100)
            ].as_ref()
        ).split(f.size());

    f.render_widget(navigation(app), rows[1]);

    f.render_widget(header(), rows[0]);
    for entry in app.entries.entries.iter() {
        f.render_widget(table(entry), rows[2])
    }
}

fn navigation(app: &app::App) -> Paragraph {
    let text: Vec<Spans> = vec![
        Spans::from(vec![
           Span::raw("< [p] "),
           Span::raw(app.current_entry().date.to_string()),
           Span::raw(" [n] >")
        ])
    ];

    Paragraph::new(text)
}

// can this not be 'static?
fn header() -> Paragraph<'static> {
    let text: Vec<Spans> = vec![
        Spans::from(vec![
           Span::raw("PTTLog")
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
