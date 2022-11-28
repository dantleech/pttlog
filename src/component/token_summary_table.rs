use anyhow::Ok;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
};

use crate::{
    model::model::{TagMeta, TagMetas},
    parser::TokenKind,
};

pub struct TokenSummaryTable<'a> {
    title: &'a str,
}

impl TokenSummaryTable<'_> {
    pub fn new<'a>(title: &'a str) -> TokenSummaryTable<'a> {
        TokenSummaryTable { title }
    }

    pub fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        area: tui::layout::Rect,
        tag_metas: &TagMetas,
    ) -> anyhow::Result<()> {
        let mut rows = vec![];
        let binding = [self.title, "Duration", "Count"];
        let headers = binding
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));

        for tag_meta in tag_metas.tag_metas.iter() {
            rows.push(Row::new([
                Cell::from((|t: &TagMeta| match tag_meta.kind {
                    TokenKind::Tag => {
                        Span::styled(format!("@{}", t.tag), Style::default().fg(Color::Green))
                    }
                    TokenKind::Prose => Span::raw(t.tag.to_owned()),
                    TokenKind::Ticket => {
                        Span::styled(format!("{}", t.tag), Style::default().fg(Color::Cyan))
                    }
                })(tag_meta)),
                Cell::from(tag_meta.duration.to_string()),
                Cell::from(tag_meta.count.to_string()),
            ]));
        }

        rows.push(Row::new([
            Cell::default(),
            Cell::default(),
            Cell::default(),
        ]));
        rows.push(Row::new([
            Cell::from(Span::styled("Total:", Style::default().fg(Color::DarkGray))),
            Cell::from(Span::raw(tag_metas.duration().to_string())),
            Cell::default(),
        ]));

        let table = Table::new(rows)
            .header(
                Row::new(headers)
                    .height(1)
                    .bottom_margin(1)
                    .style(Style::default()),
            )
            .widths(&[
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]);
        f.render_widget(table, area);
        Ok(())
    }
}
