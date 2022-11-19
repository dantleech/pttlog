use anyhow::Ok;
use tui::{
    layout::Constraint,
    style::{Color, Style},
    text::Span,
    widgets::{Cell, Row, Table},
};

use crate::{
    model::entries::{LogEntries, TagMeta},
    parser::TokenKind,
};

use super::Component;

pub struct TokenSummaryTable<'a> {
    title: &'a str,
    kind: TokenKind,
    entries: LogEntries<'a>,
}

impl TokenSummaryTable<'_> {
    pub fn new<'a>(
        title: &'a str,
        kind: TokenKind,
        entries: LogEntries<'a>,
    ) -> TokenSummaryTable<'a> {
        TokenSummaryTable {
            title,
            kind,
            entries,
        }
    }
}

impl Component for TokenSummaryTable<'_> {
    fn draw<B: tui::backend::Backend>(
        &self,
        f: &mut tui::Frame<B>,
        rect: tui::layout::Rect,
    ) -> anyhow::Result<()> {
        let mut rows = vec![];
        let binding = [self.title, "Duration", "Count"];
        let headers = binding
            .iter()
            .map(|header| Cell::from(Span::styled(*header, Style::default().fg(Color::DarkGray))));

        for tag_meta in self.entries.tag_summary(self.kind).iter() {
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

        Table::new(rows)
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
        Ok(())
    }
}
