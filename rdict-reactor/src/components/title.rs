use windows_reactor::*;

use crate::theme;

pub fn title(input_text: &str, cached: bool) -> Element {
    let mut children: Vec<Element> = vec![
        text_block(input_text)
            .font_size(theme::TITLE_FONT_SIZE)
            .font_weight(theme::TITLE_FONT_WEIGHT)
            .wrap()
            .into(),
    ];

    if cached {
        children.push(
            border(
                text_block("cached")
                    .font_size(11.0)
                    .foreground(ThemeRef::SecondaryText),
            )
            .background(ThemeRef::SubtleFill)
            .border_brush(ThemeRef::CardStroke)
            .border_thickness(Thickness::uniform(1.0))
            .corner_radius(theme::CHIP_CORNER_RADIUS)
            .padding(theme::CHIP_PADDING)
            .horizontal_alignment(HorizontalAlignment::Left)
            .into(),
        );
    }

    vstack(children).spacing(6.0).into()
}
