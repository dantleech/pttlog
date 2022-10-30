use std::collections::HashMap;

use crate::parser::Entries;
use crate::parser::TimeRange;

use super::app;

use super::parser;
use tui::layout::Margin;
use tui::style::Color;

use tui::widgets::BarChart;
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

    // By day view
    let columns = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(0)
        .constraints([
         Constraint::Percentage(50),
         Constraint::Percentage(50),
        ]).split(rows[1].inner(&Margin{ vertical: 0, horizontal: 0 }));

    f.render_widget(table(app.current_entry()), columns[0]);

    let chart_data = breakdown_chart_tag_durations(app.current_entry());
    let data = &chart_data.iter().map(|(k, v)| {
            (k.as_str(),*v as u64)
        }).collect::<Vec<(&str,u64)>>();
    let chart = BarChart::default()
        .bar_width(4)
        .data(data);

    f.render_widget(chart, columns[1])
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
                           Cell::from(description(&log.description)),
        ]));
    }

    Table::new(rows)
        .header(Row::new(headers).height(1).bottom_margin(1).style(Style::default()))
        .widths(&[
                Constraint::Min(12),
                Constraint::Min(14),
                Constraint::Percentage(100),
        ])
}

fn description(tokens: &parser::Tokens) -> Spans<'static> {
    let foo = tokens.to_vec().iter().map(|t: &parser::Token| {
        match t.kind {
            parser::TokenKind::Tag => Span::styled(
                format!("@{}", t.text.to_owned()),
                Style::default().fg(Color::Green)
            ),
            parser::TokenKind::Prose => Span::raw(t.text.to_owned()),
        }
    }).collect::<Vec<_>>();
    Spans::from(foo)
}

fn breakdown_chart_tag_durations(entry: &parser::Entry) -> HashMap<String,i16> {
    let mut counts = HashMap::<String,i16>::new();
    for entry in entry.logs.iter() {
        for tag in entry.description.tags().iter() {
            if !counts.contains_key(&tag.text) {
                counts.insert(tag.text.to_owned(), 0);
            }
            let count = counts.get_mut(&tag.text).unwrap();
            *count += entry.time.duration();
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use crate::parser::{Log, Tokens};

    use super::*;

    #[test]
    fn test_breakdown_chart_buckets() {
        let buckets = breakdown_chart_tag_durations(
            &parser::Entry{
                date: parser::Date { year: 1, month: 1, day: 1 },
                logs: vec![
                    Log{
                       time: TimeRange{ 
                           start: parser::Time { hour: 0, minute: 0 }, 
                           end: Some(parser::Time { hour: 0, minute: 10 }),
                       },
                       description: Tokens(vec![
                           parser::Token{ kind: parser::TokenKind::Prose, text: "Today is the day".to_string() },
                           parser::Token{ kind: parser::TokenKind::Tag, text: "cat1".to_string() },
                           parser::Token{ kind: parser::TokenKind::Tag, text: "cat2".to_string() },
                           parser::Token{ kind: parser::TokenKind::Tag, text: "cat2".to_string() },
                       ])
                    },
                    Log{
                       time: TimeRange{ 
                           start: parser::Time { hour: 0, minute: 0 }, 
                           end: Some(parser::Time { hour: 0, minute: 20 }),
                       },
                       description: Tokens(vec![
                           parser::Token{ kind: parser::TokenKind::Prose, text: "Today is another day".to_string() },
                           parser::Token{ kind: parser::TokenKind::Tag, text: "cat1".to_string() },
                           parser::Token{ kind: parser::TokenKind::Tag, text: "cat2".to_string() },
                       ])
                    },
                ]
            }
        );
        assert_eq!(30, buckets.get("cat1").unwrap().to_owned());
        assert_eq!(40, buckets.get("cat2").unwrap().to_owned());
    }
}
