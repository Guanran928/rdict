use iced::font;
use iced::widget::{column, container, scrollable, text};
use iced::{Element, Font, Length};
use rdict_core::parse::ja::{ToChinese, ToJapanese};

use crate::{
    components::{comparison, list_item, section, title},
    Message,
};

pub fn to_chinese(tc: &ToChinese) -> Element<'_, Message> {
    let pronunciation_col = tc.pronunciation.as_ref().map(|pr| {
        container(text(format!("[{} | {}]", pr.kana, pr.romaji)).style(text::secondary))
            .padding([4, 8])
    });

    let pos_col = tc.part_of_speech.as_ref().map(|pos| {
        container(text(pos))
            .padding([4, 8])
            .style(container::bordered_box)
    });

    let exam_col = tc
        .exam
        .as_ref()
        .map(|ex| container(text(ex).size(14).style(text::secondary)).padding([4, 8]));

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
            children = children.push(comparison(&ex.ja, &ex.zh));
        }
        Some(section("Examples", children))
    };

    scrollable(
        column![
            title(&tc.input_text),
            pronunciation_col,
            pos_col,
            meanings_col,
            exam_col,
            examples_col,
        ]
        .spacing(20)
        .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn to_japanese(te: &ToJapanese) -> Element<'_, Message> {
    let meanings_col = if te.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &te.meanings {
            let item: Element<'_, Message> = if meaning.point.is_empty() {
                list_item(text(&meaning.definition))
            } else {
                comparison(&meaning.point, &meaning.definition).into()
            };
            children = children.push(item);
        }
        Some(section("Meanings", children))
    };

    let examples_col = if te.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for ex in &te.examples {
            children = children.push(comparison(&ex.zh, &ex.ja));
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
