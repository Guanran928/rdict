use iced::Length;
use iced::widget::{column, container, scrollable, text};
use iced_material as material;
use rdict_core::parse::ko::{ToChinese, ToKorean};

use crate::{
    Message,
    components::{comparison, list_item, section, title},
};

pub fn to_chinese(tc: &ToChinese) -> material::Element<'_, Message> {
    let meanings_col = if tc.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &tc.meanings {
            let mut inner = column![].spacing(4);
            if let Some(pos) = &meaning.part_of_speech {
                inner = inner.push(
                    container(text(pos))
                        .padding([4, 8])
                        .style(material::style::container::outlined),
                );
            }
            for definition in &meaning.definitions {
                inner = inner.push(list_item(text(definition)));
            }
            if let Some(ex) = &meaning.example {
                inner = inner.push(text(&ex.ko).size(14).style(material::text::surface_variant));
                inner = inner.push(text(&ex.zh).size(14).style(material::text::surface_variant));
            }
            children = children.push(inner);
        }
        Some(section("Meanings", children))
    };

    scrollable(
        column![title(&tc.input_text), meanings_col,]
            .spacing(20)
            .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn to_korean(te: &ToKorean) -> material::Element<'_, Message> {
    let meanings_col = if te.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &te.meanings {
            let mut inner = column![].spacing(4);
            if let Some(pos) = &meaning.part_of_speech {
                inner = inner.push(
                    container(text(pos))
                        .padding([4, 8])
                        .style(material::style::container::outlined),
                );
            }
            for definition in &meaning.definitions {
                inner = inner.push(list_item(text(definition)));
            }
            children = children.push(inner);
        }
        Some(section("Meanings", children))
    };

    let examples_col = if te.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for ex in &te.examples {
            children = children.push(comparison(&ex.zh, &ex.ko));
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
