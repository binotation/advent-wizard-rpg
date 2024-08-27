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
    text: &'a str,
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

        buf.set_string(
            area.left() + area.width / 2 - self.text.len() as u16 / 2,
            area.top() + area.height / 2,
            self.text,
            Style::default().bold(),
        );
    }
}

impl<'a> CenterPosition<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn text(mut self, text: &'a str) -> CenterPosition<'a> {
        self.text = text;
        self
    }
}
