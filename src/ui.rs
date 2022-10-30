use crate::parser::TimeRange;

use super::app;

use super::parser;
use tui::style::Color;

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
           Span::raw(format!("{}/{} ", app.current_entry_number().to_string(), app.entry_count().to_string())),
           Span::styled("[p] ", Style::default().fg(Color::Green)),
           Span::styled(app.current_entry().date.to_string(), Style::default().fg(Color::LightBlue)),
           Span::styled(" [n]", Style::default().fg(Color::Green)),
           Span::raw(" "),
           Span::raw(app.current_entry().duration_total_as_string()),
        ])
    ];

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
                           Cell::from(
                              (|time: &TimeRange| {
                                  if time.end.is_none() {
                                      return Spans::from(vec![Span::raw(time.start.to_string())]);
                                  }
                                  Spans::from(vec![
                                    Span::raw(time.start.to_string()),
                                    Span::styled("-", Style::default().fg(Color::DarkGray)),
                                    Span::styled(time.end.unwrap().to_string(), Style::default().fg(Color::DarkGray)),
                                  ])
                              })(&log.time)
                           ),
                           Cell::from(Spans::from(vec![
                              Span::raw(log.duration_as_string()),
                              Span::styled(format!(" {:.2}%", log.as_percentage(entry_duration)), Style::default().fg(Color::DarkGray)),
                           ])),
                           Cell::from(log.description.to_string()),
        ]));
    }

    Table::new(rows)
        .header(Row::new(headers).height(1).bottom_margin(1).style(Style::default()))
        .widths(&[
                Constraint::Min(12),
                Constraint::Min(14),
                Constraint::Percentage(33),
        ])
        .block(Block::default().borders(Borders::ALL).title(entry.date_object().format("%A %e %B, %Y").to_string()))
}
