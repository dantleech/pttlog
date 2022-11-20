use anyhow::{Error, Result};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

use crate::model::entries::LogDays;

use super::component::day::Day;

use self::config::{Config, KeyMap};
use super::parser;
pub mod config;
pub mod loader;

pub struct App<'a> {
    pub notification: Notification,
    loader: Box<dyn loader::Loader + 'a>,
    pub log_days: LogDays,
    pub day: Day<'a>,
}

impl App<'_> {
    pub fn new<'a>(loader: Box<dyn loader::Loader + 'a>, _config: &'a Config) -> App<'a> {
        let log_days = LogDays::new(parser::Entries {
            entries: vec![parser::Entry::placeholder()],
        });
        App {
            log_days,
            loader,
            notification: Notification {
                notification: "".to_string(),
                lifetime: 0,
            },
            day: Day::new(),
        }
    }

    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), Error> {
        self.notification.tick();

        let rows = Layout::default()
            .margin(0)
            .constraints([Constraint::Length(1), Constraint::Min(4)].as_ref())
            .split(f.size());

        f.render_widget(navigation(), rows[0]);

        self.day.draw(f, rows[1], &self.log_days)?;

        if self.notification.should_display() {
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
        Ok(())
    }

    pub fn notify(&mut self, message: String, lifetime: i16) {
        self.notification.notification = message;
        self.notification.lifetime = lifetime;
    }

    pub fn reload(&mut self) {
        self.log_days = LogDays::new(self.loader.load());
    }

    pub(crate) fn handle(&mut self, key: KeyMap) {
        self.day.handle(key);
    }
}

#[derive(Debug)]
pub struct Notification {
    pub notification: String,
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
        Span::raw("eload"),
        Span::styled(" [q]", Style::default().fg(Color::Green)),
        Span::raw("uit"),
    ])];

    Paragraph::new(text)
}
