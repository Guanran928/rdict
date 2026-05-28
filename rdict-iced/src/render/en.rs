use iced::font;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Font, Length};
use rdict_core::model::Voice;
use rdict_core::parse::en::{ToChinese, ToEnglish};

use crate::{
    Message,
    components::{comparison, list_item, section, title},
    render,
};

pub fn to_chinese<'a>(tc: &'a ToChinese, voices: &'a [Voice]) -> Element<'a, Message> {
    // Pronunciation Layout

    let build_accent = |label: &'static str, value: &str, voice: Option<&Voice>| {
        row![
            text(label).font(Font {
                weight: font::Weight::Bold,
                ..Font::default()
            }),
            " ",
            render::pronunciation(value.to_owned(), voice)
        ]
    };

    let mut accents = Vec::new();
    if let Some(uk) = &tc.pronunciation.uk {
        accents.push(build_accent("英", uk, voices.first()).into());
    }
    if let Some(us) = &tc.pronunciation.us {
        accents.push(build_accent("美", us, voices.get(1)).into());
    }

    let pronunciation_col = (!accents.is_empty()).then(|| row(accents).spacing(15));

    // Meanings Layout
    let meanings_col = if tc.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &tc.meanings {
            let mut definitions_col = column![].spacing(2);
            if let Some(p) = &meaning.part_of_speech {
                definitions_col = definitions_col.push(
                    container(text(p))
                        .padding([4, 8])
                        .style(container::bordered_box),
                );
            }
            for definition in &meaning.definitions {
                definitions_col = definitions_col.push(list_item(text(definition)));
            }
            children = children.push(definitions_col);
        }
        Some(section("Meanings", children))
    };

    // Examples Layout
    let examples_col = if tc.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for example in &tc.examples {
            children = children.push(comparison(&example.en, &example.zh));
        }
        Some(section("Examples", children))
    };

    // Exams Layout
    let exams_col = if tc.exams.is_empty() {
        None
    } else {
        let mut children = row![].spacing(2);
        for (i, exam) in tc.exams.iter().enumerate() {
            // HACK: ASCII
            if i != 0 {
                children = children.push(text("|").style(text::secondary))
            }

            children = children.push(text(exam));
        }
        Some(section("Exams", children))
    };

    scrollable(
        column![
            title(&tc.input_text),
            pronunciation_col,
            meanings_col,
            examples_col,
            exams_col,
        ]
        .spacing(20)
        .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn to_english(te: &ToEnglish) -> Element<'_, Message> {
    // Meanings Layout
    let meanings_col = if te.meanings.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for meaning in &te.meanings {
            children = children.push(list_item(text(meaning)));
        }
        Some(section("Meanings", children))
    };

    // Examples Layout
    let examples_col = if te.examples.is_empty() {
        None
    } else {
        let mut children = column![].spacing(10);
        for example in &te.examples {
            children = children.push(comparison(&example.zh, &example.en));
        }
        Some(section("Examples", children))
    };

    scrollable(
        column![title(&te.input_text), meanings_col, examples_col]
            .spacing(20)
            .padding(10),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
