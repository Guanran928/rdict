use crate::Message;
use iced::{
    Font, font,
    widget::{column, container, text},
};
use iced_material as material;

pub fn section<'a>(
    title: &'a str,
    children: impl Into<material::Element<'a, Message>>,
) -> material::Element<'a, Message> {
    column![
        text(title)
            .style(material::text::surface_variant)
            .font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
        material::widget::rule::horizontal_full_width(),
        container(children)
    ]
    .spacing(10)
    .into()
}
