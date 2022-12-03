use anyhow::Error;
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use crate::{
    app::config::{Config, Key as PtKey},
    parser::filter::{parse_filter, Filter as ParserFilter},
    ui::centered_rect_absolute,
};

pub struct Filter<'a> {
    pub textarea: TextArea<'a>,
    pub visible: bool,
    pub valid: bool,
    pub filter: Option<ParserFilter>,
}

impl Filter<'_> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        Filter {
            textarea,
            visible: false,
            valid: false,
            filter: None,
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

        if key.event.code == KeyCode::Enter {
            self.visible = false;
            return;
        }

        self.textarea.input(key.event);
        match parse_filter(&self.textarea.lines()[0], &Config::empty()) {
            Ok(ok) => {
                self.valid = true;
                self.filter = Some(ok);
            }
            Err(_err) => {
                self.valid = false;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ui::stream_input_to;

    use super::*;
    #[test]
    pub fn disable_visibility_on_enter() {
        let mut filter = Filter::new();
        filter.visible = true;
        filter.handle(&PtKey::for_key_code(KeyCode::Enter));
        assert_eq!(false, filter.visible);
    }

    #[test]
    pub fn parses_input() {
        let mut filter = Filter::new();
        filter.visible = true;
        stream_input_to("@phpactor @foobar".to_string(), |key| filter.handle(&key));
        assert_eq!("@phpactor @foobar", filter.textarea.lines()[0]);
        assert_eq!(true, filter.valid);

        assert_eq!(2, filter.filter.unwrap().criterias.len())
    }
}
