use crate::Message;
use iced::{
    Font, font,
    widget::{column, text},
};
use iced_material as material;

pub fn comparison<'a>(first: &'a str, second: &'a str) -> material::Element<'a, Message> {
    column![
        text(first).style(material::text::surface).font(Font {
            weight: font::Weight::Medium,
            ..Font::default()
        }),
        text(second).size(14).style(material::text::surface_variant),
    ]
    .spacing(5)
    .into()
}
