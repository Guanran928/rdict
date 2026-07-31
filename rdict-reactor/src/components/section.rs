use windows_reactor::*;

use crate::theme;

pub fn section(title: &str, children: impl Into<Element>) -> Element {
    vstack((
        text_block(title)
            .font_size(14.0)
            .semibold()
            .foreground(ThemeRef::SecondaryText),
        border(Element::Empty)
            .background(ThemeRef::DividerStroke)
            .height(1.0),
        children.into(),
    ))
    .spacing(theme::SECTION_SPACING)
    .into()
}
