use crate::Message;
use iced::{
    Alignment,
    widget::{row, text},
};
use iced_material as material;

pub fn list_item<'a>(
    content: impl Into<material::Element<'a, Message>>,
) -> material::Element<'a, Message> {
    row![
        text("•").size(20).style(material::text::surface_variant),
        content.into()
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
