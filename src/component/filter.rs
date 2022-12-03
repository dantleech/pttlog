use anyhow::Error;
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use crate::{app::config::Key as PtKey, ui::centered_rect_absolute};

pub struct Filter<'a> {
    pub textarea: TextArea<'a>,
    pub visible: bool,
}

impl Filter<'_> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        Filter {
            textarea,
            visible: false,
        }
    }
    pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), Error> {
        let area = centered_rect_absolute(64, 3, f.size());

        if self.visible == false {
            return Ok(());
        }

        self.textarea
            .set_block(Block::default().borders(Borders::ALL).title("Filter"));
        self.textarea
            .set_style(Style::default().fg(Color::LightGreen));

        f.render_widget(self.textarea.widget(), area);
        Ok(())
    }

    pub(crate) fn handle(&mut self, key: &PtKey) {
        if !self.visible {
            return;
        }

        match key.event.code {
            KeyCode::Enter => {
                self.visible = false;
            }
            _ => {
                self.textarea.input(key.event);
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn disable_visibility_on_enter() {
        let mut filter = Filter::new();
        filter.visible = true;
        filter.handle(&PtKey::for_key_code(KeyCode::Enter));
        assert_eq!(false, filter.visible);
    }
}
