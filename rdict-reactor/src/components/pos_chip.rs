use windows_reactor::*;

use crate::theme;

pub fn pos_chip(pos: &str) -> Element {
    border(text_block(pos).font_size(12.0))
        .background(ThemeRef::SubtleFill)
        .border_brush(ThemeRef::CardStroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(theme::CHIP_CORNER_RADIUS)
        .padding(theme::CHIP_PADDING)
        .horizontal_alignment(HorizontalAlignment::Left)
        .into()
}
