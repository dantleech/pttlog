use anyhow::{Error, Result};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    component::{filter::Filter, interval_view::IntervalView},
    model::{model::LogDays, time::TimeFactory},
    parser::timesheet::Entry,
};

use super::component::day::Day;

use self::config::{Config, Key, KeyName};
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
    pub filtered: LogDays,
    day: Day<'a>,
    week: IntervalView<'a>,
    year: IntervalView<'a>,
    view: AppView,
    filter: Filter<'a>,
}

impl App<'_> {
    pub fn new<'a>(
        loader: Box<dyn loader::Loader + 'a>,
        _config: &'a Config,
        time_factory: &'a dyn TimeFactory,
        now: &'a NaiveDateTime,
    ) -> App<'a> {
        let log_days = LogDays::new(vec![Entry::placeholder()]);
        App {
            filtered: log_days.clone(),
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
                time_factory,
                NaiveDate::from_isoywd(now.year(), now.iso_week().week(), chrono::Weekday::Mon),
                Duration::weeks(1),
            ),
            year: IntervalView::new(
                time_factory,
                NaiveDate::from_ymd(now.year() - 1, now.month(), now.day()),
                Duration::days(365),
            ),
            filter: Filter::new(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), Error> {
        self.notification.tick();
        self.apply_filter();

        let rows = Layout::default()
            .margin(0)
            .constraints([Constraint::Length(1), Constraint::Min(4)].as_ref())
            .split(f.size());

        f.render_widget(navigation(), rows[0]);

        match self.view {
            AppView::Day => self.day.draw(f, rows[1], &self.filtered)?,
            AppView::Week => self.week.draw(f, rows[1], &self.filtered)?,
            AppView::Year => self.year.draw(f, rows[1], &self.filtered)?,
        };

        self.filter.draw(f)?;

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

    pub fn apply_filter(&mut self) {
        if let Some(filter) = &self.filter.filter {
            self.filtered = self.log_days.filter(filter);
            return;
        }
        self.filtered = self.log_days.clone()
    }

    fn set_view(&mut self, view: AppView) {
        self.view = view
    }

    pub(crate) fn handle(&mut self, key: Key) {
        if self.filter.visible == true {
            self.filter.handle(&key);
            return;
        }
        match key.name {
            KeyName::ToggleFilter => self.filter.visible = true,
            KeyName::DayView => self.set_view(AppView::Day),
            KeyName::WeekView => self.set_view(AppView::Week),
            KeyName::YearView => self.set_view(AppView::Year),
            KeyName::Reload => {
                self.reload();
                self.notify("reloaded timesheet".to_string(), 2);
            }
            _ => {
                match self.view {
                    AppView::Day => self.day.handle(&key.name),
                    AppView::Week => self.week.handle(&key.name),
                    AppView::Year => self.year.handle(&key.name),
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

#[cfg(test)]
mod test {
    use crate::{model::time::FrozenTimeFactory, parser::timesheet::Entries};

    use super::{loader::FuncLoader, *};

    #[test]
    pub fn last_day_of_month() {
        App::new(
            FuncLoader::new(Box::new(|| Entries { entries: vec![] })),
            &Config::empty(),
            &FrozenTimeFactory::new(2022, 1, 1, 12, 0),
            &NaiveDate::from_ymd(2022, 11, 30).and_hms(10, 1, 1),
        );
    }
}
