use super::app;

use tui::layout::Alignment;
use tui::layout::Margin;
use tui::style::Color;

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::Style,
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

pub fn layout<B: Backend>(f: &mut Frame<B>, app: &mut app::App) {
    let rows = Layout::default()
        .margin(0)
        .constraints([Constraint::Length(1), Constraint::Min(4)].as_ref())
        .split(f.size());

    f.render_widget(navigation(), rows[0]);

    app.day.draw(f, rows[1], &app.log_days);

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
