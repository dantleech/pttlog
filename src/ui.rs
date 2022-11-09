use crate::app::entry_view::EntryView;
use crate::app::entry_view::TagMeta;
use crate::app::entry_view::TimeRangeView;
use crate::parser::TokenKind;

use super::app;

use super::parser;

use nom::ToUsize;

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

pub fn layout<B: Backend>(f: &mut Frame<B>, app: &mut app::App) {
    let rows = Layout::default()
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Min(4)].as_ref())
        .split(f.size());

    f.render_widget(navigation(app), rows[0]);

    let columns = Layout::default()
        .direction(tui::layout::Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[1].inner(&Margin {
            vertical: 2,
            horizontal: 2,
        }));

    let current_entry = app.current_entry();
    let container = Block::default()
        .borders(Borders::ALL)
        .title(current_entry.date().to_verbose_string());

    f.render_widget(log_table(&app, &current_entry), columns[0]);

    let summary_rows = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Min(2)])
        .split(columns[1]);

    f.render_widget(
        token_summmary_table(&current_entry, TokenKind::Tag, "Tag"),
        summary_rows[0],
    );
    f.render_widget(
        token_summmary_table(&current_entry, TokenKind::Ticket, "Ticket"),
        summary_rows[1],
    );

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

fn navigation<'a>(app: &'a app::App) -> Paragraph<'a> {
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

fn token_summmary_table<'a>(entry: &'a EntryView, kind: TokenKind, title: &'a str) -> Table<'a> {
    let mut rows = vec![];
    let binding = [title, "Duration", "Count"];
    let headers = binding
        .iter()
        .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));

    for tag_meta in entry.tag_summary(kind).iter() {
        rows.push(Row::new([
            Cell::from((|t: &TagMeta| match tag_meta.kind {
                parser::TokenKind::Tag => {
                    Span::styled(format!("@{}", t.tag), Style::default().fg(Color::Green))
                }
                parser::TokenKind::Prose => Span::raw(t.tag.to_owned()),
                parser::TokenKind::Ticket => {
                    Span::styled(format!("{}", t.tag), Style::default().fg(Color::Cyan))
                }
            })(tag_meta)),
            Cell::from(tag_meta.duration.to_string()),
            Cell::from(tag_meta.count.to_string()),
        ]));
    }

    Table::new(rows)
        .header(
            Row::new(headers)
                .height(1)
                .bottom_margin(1)
                .style(Style::default()),
        )
        .widths(&[
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
}

pub fn log_table<'a>(app: &app::App, entry: &'a EntryView) -> Table<'a> {
    let mut rows = vec![];
    let headers = ["Time", "Duration", "Description", ""]
        .iter()
        .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));
    let _duration_total = entry.duration_total();

    for log in entry.logs().iter() {
        rows.push(Row::new([
            Cell::from((|time_range: &TimeRangeView| {
                // 1. if today and end time not set show "now"
                // 2. Show clock animation
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
                    (|| {
                        if time_range.ongoing {
                            return Span::raw(format!(" {}", clock_animation(app.iteration)));
                        }
                        Span::raw("")
                    })(),
                ])
            })(&log.time_range())),
            Cell::from((|range: &TimeRangeView| -> Spans {
                Spans::from(vec![
                    Span::raw(range.duration().to_string()),
                    Span::styled(
                        format!(
                            " {:.2}%",
                            log.percentage_of_day(entry.duration_total().num_minutes())
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
        Cell::from(Span::raw(entry.duration_total().to_string())),
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
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Percentage(65),
        ])
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

fn clock_animation(iteration: u8) -> String {
    let faces: Vec<&str> = vec![
        "🕐", "🕜", "🕑", "🕝", "🕒", "🕞", "🕓", "🕟", "🕔", "🕠", "🕕", "🕡", "🕖", "🕢", "🕗",
        "🕣", "🕘", "🕤", "🕙", "🕥", "🕚", "🕦", "🕛", "🕧",
    ];

    faces[iteration.to_usize() % faces.len()].to_string()
}
