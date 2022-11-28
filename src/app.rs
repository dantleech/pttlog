use anyhow::{Error, Result};
use chrono::{Datelike, Duration, Local, NaiveDate};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{component::interval_view::IntervalView, model::model::LogDays};

use super::component::day::Day;

use self::config::{Config, KeyMap};
use super::parser;
pub mod config;
pub mod loader;

enum AppView {
    Day,
    Week,
    Year,
}

pub struct App<'a> {
    pub notification: Notification,
    loader: Box<dyn loader::Loader + 'a>,
    pub log_days: LogDays,
    day: Day<'a>,
    week: IntervalView<'a>,
    year: IntervalView<'a>,
    view: AppView,
}

impl App<'_> {
    pub fn new<'a>(loader: Box<dyn loader::Loader + 'a>, _config: &'a Config) -> App<'a> {
        let now = Local::now().naive_local();
        let log_days = LogDays::new(vec![parser::Entry::placeholder()]);
        App {
            log_days,
            loader,
            notification: Notification {
                level: NotificationLevel::Info,
                notification: "".to_string(),
                lifetime: 0,
            },
            day: Day::new(),
            view: AppView::Day,
            week: IntervalView::new(
                NaiveDate::from_isoywd(now.year(), now.iso_week().week(), chrono::Weekday::Mon),
                Duration::weeks(1),
            ),
            year: IntervalView::new(
                NaiveDate::from_ymd(now.year() - 1, now.month(), now.day() + 1),
                Duration::days(365),
            ),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), Error> {
        self.notification.tick();

        let rows = Layout::default()
            .margin(0)
            .constraints([Constraint::Length(1), Constraint::Min(4)].as_ref())
            .split(f.size());

        f.render_widget(navigation(), rows[0]);

        match self.view {
            AppView::Day => self.day.draw(f, rows[1], &self.log_days)?,
            AppView::Week => self.week.draw(f, rows[1], &self.log_days)?,
            AppView::Year => self.year.draw(f, rows[1], &self.log_days)?,
        };

        if self.notification.should_display() {
            match self.notification.level {
                NotificationLevel::Info => {
                    let text: Vec<Spans> = vec![Spans::from(vec![Span::raw(
                        &self.notification.notification,
                    )])];

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
                NotificationLevel::Error => {
                    let mut span = Span::raw(&self.notification.notification);
                    span.style = Style::default().fg(Color::Red).bg(Color::Black);
                    let text: Vec<Spans> = vec![Spans::from(vec![span])];

                    let notification = Paragraph::new(text)
                        .alignment(Alignment::Center)
                        .block(Block::default().title("Error").borders(Borders::ALL));

                    f.render_widget(
                        Clear,
                        rows[1].inner(&Margin {
                            vertical: 10,
                            horizontal: 10,
                        }),
                    );
                    let layout = Layout::default()
                        .margin(10)
                        .constraints(
                            [
                                Constraint::Percentage(33),
                                Constraint::Percentage(33),
                                Constraint::Percentage(33),
                            ]
                            .as_ref(),
                        )
                        .split(rows[1]);
                    f.render_widget(notification, layout[1])
                }
            }
        }
        Ok(())
    }

    pub fn notify(&mut self, message: String, lifetime: i16) {
        self.notification.notification = message;
        self.notification.lifetime = lifetime;
        self.notification.level = NotificationLevel::Info;
    }

    pub fn error(&mut self, message: String, lifetime: i16) {
        self.notification.notification = message;
        self.notification.lifetime = lifetime;
        self.notification.level = NotificationLevel::Error;
    }

    pub fn reload(&mut self) {
        let entries = match self.loader.load() {
            Ok(ok) => ok.entries,
            Err(err) => {
                self.error(err.to_string(), 4);
                return;
            }
        };

        self.log_days = LogDays::new(entries);
    }

    fn set_view(&mut self, view: AppView) {
        self.view = view
    }

    pub(crate) fn handle(&mut self, key: KeyMap) {
        match key {
            KeyMap::DayView => self.set_view(AppView::Day),
            KeyMap::WeekView => self.set_view(AppView::Week),
            KeyMap::YearView => self.set_view(AppView::Year),
            _ => {
                match self.view {
                    AppView::Day => self.day.handle(&key),
                    AppView::Week => self.week.handle(&key),
                    AppView::Year => self.year.handle(&key),
                };
            }
        };
    }
}

#[derive(Debug)]
enum NotificationLevel {
    Info,
    Error,
}

#[derive(Debug)]
pub struct Notification {
    pub notification: String,
    level: NotificationLevel,
    lifetime: i16,
}

impl Notification {
    fn tick(&mut self) {
        if self.lifetime > 0 {
            self.lifetime -= 1
        }
    }
    pub fn should_display(&self) -> bool {
        return self.lifetime > 0;
    }
}

fn navigation<'a>() -> Paragraph<'a> {
    let text: Vec<Spans> = vec![Spans::from(vec![
        Span::styled("[p]", Style::default().fg(Color::Green)),
        Span::raw("rev "),
        Span::styled("[n]", Style::default().fg(Color::Green)),
        Span::raw("ext "),
        Span::styled("[r]", Style::default().fg(Color::Green)),
        Span::raw("eload "),
        Span::styled("[d]", Style::default().fg(Color::Green)),
        Span::raw("ay "),
        Span::styled("[w]", Style::default().fg(Color::Green)),
        Span::raw("eek "),
        Span::styled("[y]", Style::default().fg(Color::Green)),
        Span::raw("ear "),
        Span::styled("[q]", Style::default().fg(Color::Green)),
        Span::raw("uit"),
    ])];

    Paragraph::new(text)
}
