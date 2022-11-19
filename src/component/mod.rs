mod day;
mod log_table;
mod token_summary_table;
use anyhow::Result;
use tui::{backend::Backend, layout::Rect, Frame};

pub trait Component {
    fn next(&mut self) {}
    fn prev(&mut self) {}

    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> Result<()>;
}
