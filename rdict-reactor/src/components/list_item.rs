use windows_reactor::*;

use crate::theme;

pub fn list_item(content: impl Into<Element>) -> Element {
    hstack((
        border(Element::Empty)
            .background(ThemeRef::SecondaryText)
            .corner_radius(theme::BULLET_SIZE / 2.0)
            .width(theme::BULLET_SIZE)
            .height(theme::BULLET_SIZE)
            .vertical_alignment(VerticalAlignment::Center),
        content.into(),
    ))
    .spacing(10.0)
    .into()
}
