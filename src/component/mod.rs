mod day;
use anyhow::Result;
use tui::{backend::Backend, Frame, layout::Rect};

pub trait Component {
    fn next(&mut self) {
    }
    fn prev(&mut self) {
    }

	fn draw<B: Backend>(
		&self,
		f: &mut Frame<B>,
		rect: Rect,
	) -> Result<()>;
}
