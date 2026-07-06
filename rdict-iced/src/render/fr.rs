use iced::Length;
use iced::widget::{column, container, scrollable, text};
use iced_material as material;
use rdict_core::parse::fr::{ToChinese, ToFrench};

use crate::{
    Message,
    components::{comparison, list_item, section, title},
};

pub fn to_chinese(tc: &ToChinese) -> material::Element<'_, Message> {
    let pronunciation_col = tc.pronunciation.as_ref().map(|ph| {
        container(text(format!("[{ph}]")).style(material::text::surface_variant)).padding([4, 8])
    });

    let meanings_col = if tc.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &tc.meanings {
            children = children.push(list_item(text(meaning)));
        }
        Some(section("Meanings", children))
    };

    let examples_col = if tc.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for ex in &tc.examples {
            children = children.push(comparison(&ex.fr, &ex.zh));
        }
        Some(section("Examples", children))
    };

    scrollable(
        column![
            title(&tc.input_text),
            pronunciation_col,
            meanings_col,
            examples_col,
        ]
        .spacing(20)
        .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn to_french(te: &ToFrench) -> material::Element<'_, Message> {
    let meanings_col = if te.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &te.meanings {
            children = children.push(list_item(text(meaning)));
        }
        Some(section("Meanings", children))
    };

    let examples_col = if te.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for ex in &te.examples {
            children = children.push(comparison(&ex.zh, &ex.fr));
        }
        Some(section("Examples", children))
    };

    scrollable(
        column![title(&te.input_text), meanings_col, examples_col,]
            .spacing(20)
            .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
