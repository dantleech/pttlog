use tui::{layout::Margin, style::Style, widgets::{Block, Borders, Paragraph}};

use crate::model::model::LogContext;

pub struct EpochListComponent {}

impl EpochListComponent {
    pub fn new() -> Self
    {
        EpochListComponent{}
    }

    pub fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        context: &LogContext,
    ) -> anyhow::Result<()> {
        let block = Block::default().style(Style::default().bg(tui::style::Color::LightBlue));

        let mut lines = vec![];
        for epoch in &context.epochs.epochs {
            lines.push(format!(
                "prefix: {:?}, tags: {:?}, rate: {:?} per hour",
                epoch.ticket_prefix,
                epoch.tags.join(","),
                epoch.rate().to_string(),
            ))
        }
        f.render_widget(block, area);
        f.render_widget(Paragraph::new(lines.join("\n")), area);
        Ok(())
    }
}

