use anyhow::Error;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::Style,
    Frame,
};
use tui_textarea::TextArea;

use crate::app::config::Key;

pub struct Filter<'a> {
    pub textarea: TextArea<'a>,
}

impl Filter<'_> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        Filter { textarea }
    }
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), Error> {
        let layout = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_slice())
            .split(f.size());

        f.render_widget(self.textarea.widget(), layout[0]);
        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &Key) {
        self.textarea.input(key.event);
    }
}
