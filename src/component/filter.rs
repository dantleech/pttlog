use anyhow::Error;
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
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
    pub original_filter: Vec<String>,
    pub config: &'a Config,
}

impl Filter<'_> {
    pub fn new<'a>(config: &'a Config) -> Filter<'a> {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        Filter {
            textarea,
            visible: false,
            valid: false,
            filter: None,
            config,
            original_filter: [].to_vec(),
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

        f.render_widget(Clear, area);
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

        if key.event.code == KeyCode::Esc {
            self.visible = false;
            self.textarea = TextArea::new(self.original_filter.to_vec());
            self.textarea.set_cursor_line_style(Style::default());
        }

        self.textarea.input(key.event);
        match parse_filter(&self.textarea.lines()[0], self.config) {
            Ok(ok) => {
                self.valid = true;
                self.filter = Some(ok);
            }
            Err(_err) => {
                self.valid = false;
            }
        }
    }

    pub(crate) fn show(&mut self) {
        self.visible = true;
        self.original_filter = self.textarea.lines().to_vec().clone();
    }
}

#[cfg(test)]
mod test {
    use crate::{ui::stream_input_to, app::config::Project};

    use super::*;
    #[test]
    pub fn disable_visibility_on_enter() {
        let binding = Config::empty();
        let mut filter = Filter::new(&binding);
        filter.visible = true;
        filter.handle(&PtKey::for_key_code(KeyCode::Enter));
        assert_eq!(false, filter.visible);
    }

    #[test]
    pub fn reset_state_on_esc() {
        let binding = Config::empty();
        let mut filter = Filter::new(&binding);
        filter.visible = true;
        filter.handle(&PtKey::for_key_code(KeyCode::Esc));
        assert_eq!(false, filter.visible);
    }

    #[test]
    pub fn parses_input() {
        let binding = Config::empty();
        let mut filter = Filter::new(&binding);
        filter.visible = true;
        stream_input_to("@phpactor @foobar".to_string(), |key| filter.handle(&key));
        assert_eq!("@phpactor @foobar", filter.textarea.lines()[0]);
        assert_eq!(true, filter.valid);

        assert_eq!(2, filter.filter.unwrap().criterias.len())
    }

    #[test]
    pub fn parses_input_with_ticket() {
        let config = Config {
            projects: vec![Project {
                name: "myproject".to_string(),
                ticket_prefix: "PROJECT-".to_string(),
                tags: vec![],
            }],
        };
        let mut filter = Filter::new(&config);
        filter.visible = true;
        stream_input_to("PROJECT-123".to_string(), |key| filter.handle(&key));
        assert_eq!("Ticket(PROJECT-123)", filter.filter.unwrap().to_string());
    }
}
