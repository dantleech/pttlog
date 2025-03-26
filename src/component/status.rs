use tui::{widgets::Paragraph, text::Spans};

use crate::app::App;

pub struct Status {
}

impl Status {
    pub fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        app: &App,
    ) -> anyhow::Result<()> {
        if !self.display(app) {
            return Ok(());
        }

        if let Some(filter) = &app.filter.filter {
            f.render_widget(Paragraph::new(Spans::from(filter.to_string())), area);
        }
        Ok(())
    }

    pub fn display(&self, app: &App) -> bool {
        if let Some(filter) = &app.filter.filter {
            return !filter.criterias.is_empty();
        }

        false
    }

    pub(crate) fn new() -> Status {
        Status{}
    }
}
