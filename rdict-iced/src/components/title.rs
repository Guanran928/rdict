use crate::Message;
use iced::{Font, font, widget::text};
use iced_material as material;

pub fn title(input_text: &str) -> material::Element<'_, Message> {
    text(input_text)
        .size(40)
        .style(material::text::surface)
        .font(Font {
            weight: font::Weight::ExtraBold,
            ..Font::default()
        })
        .into()
}
