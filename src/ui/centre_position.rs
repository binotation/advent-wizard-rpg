//! Custom widget for centered text.
//! Source: https://github.com/fdehau/tui-rs/issues/396#issuecomment-1430447664

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Stylize,
    style::Style,
    widgets::{block::Block, Widget},
};

#[derive(Default)]
pub struct CenterPosition<'a> {
    block: Option<Block<'a>>,
    text: String,
    unavailable: bool,
}

impl<'a> Widget for CenterPosition<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let text_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if text_area.height < 1 {
            return;
        }

        let style = if self.unavailable {
            Style::default().bold().crossed_out().red()
        } else {
            Style::default().bold()
        };

        buf.set_string(
            area.left() + area.width / 2 - self.text.len() as u16 / 2,
            area.top() + area.height / 2,
            self.text,
            style,
        );
    }
}

impl<'a> CenterPosition<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn text(mut self, text: String) -> CenterPosition<'a> {
        self.text = text;
        self
    }

    pub fn unavailable(mut self) -> CenterPosition<'a> {
        self.unavailable = true;
        self
    }
}
