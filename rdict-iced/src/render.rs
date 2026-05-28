pub mod en;
pub mod fr;
pub mod ja;
pub mod ko;

use crate::Message;
use iced::Element;
use iced::widget::{button, text};
use rdict_core::model::Voice;

pub(crate) fn pronunciation<'a>(value: String, voice: Option<&Voice>) -> Element<'a, Message> {
    let label = text(format!("[{value}]")).style(text::secondary);

    if let Some(voice) = voice {
        button(label)
            .padding(0)
            .style(iced::widget::button::text)
            .on_press(Message::PlayVoice(voice.clone()))
            .into()
    } else {
        label.into()
    }
}
