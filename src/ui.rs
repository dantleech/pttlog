use super::parser;
use tui::{widgets::{Table, Block, Row, Borders, Cell}, layout::Constraint, style::Style};

pub fn table(entries: &parser::Entries) -> Table {
    let mut rows = vec![];
    let headers = ["Start", "Duration", "Description"]
        .iter()
        .map(|header| Cell::from(*header));

    for entry in entries.entries.iter() {
        for log in entry.logs.iter() {
            rows.push(Row::new([
                Cell::from(log.time.to_string()),
                Cell::from(log.duration_as_string()),
                Cell::from(log.description.as_str()),
            ]));
        }
    }

    Table::new(rows)
        .header(Row::new(headers).height(1).bottom_margin(1).style(Style::default()))
        .widths(&[
            Constraint::Min(7),
            Constraint::Min(8),
            Constraint::Percentage(33),
        ])
        .block(Block::default().borders(Borders::ALL).title("Timesheet"))
}
