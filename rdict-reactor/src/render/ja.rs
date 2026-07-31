use rdict_core::parse::ja::{ToChinese, ToJapanese};
use windows_reactor::*;

use crate::components::{comparison, list_item, pos_chip, section, title};
use crate::theme;

pub fn to_chinese(tc: &ToChinese, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&tc.input_text, is_cached));

    if let Some(pronunciation) = &tc.pronunciation {
        children.push(
            border(
                text_block(format!(
                    "[{} | {}]",
                    pronunciation.kana, pronunciation.romaji
                ))
                .foreground(ThemeRef::SecondaryText),
            )
            .padding(theme::CHIP_PADDING)
            .into(),
        );
    }

    if let Some(pos) = &tc.part_of_speech {
        children.push(pos_chip(pos));
    }

    if let Some(exam) = &tc.exam {
        children.push(
            border(
                text_block(exam)
                    .font_size(14.0)
                    .foreground(ThemeRef::SecondaryText),
            )
            .padding(theme::CHIP_PADDING)
            .into(),
        );
    }

    // Meanings Layout
    if !tc.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &tc.meanings {
            meanings.push(list_item(text_block(meaning).wrap()));
        }
        children.push(section(
            "Meanings",
            vstack(meanings).spacing(theme::SECTION_SPACING),
        ));
    }

    // Examples Layout
    if !tc.examples.is_empty() {
        let mut examples: Vec<Element> = Vec::new();
        for example in &tc.examples {
            examples.push(comparison(&example.ja, &example.zh));
        }
        children.push(section(
            "Examples",
            vstack(examples).spacing(theme::SECTION_SPACING),
        ));
    }

    scroll_viewer(
        vstack(children)
            .spacing(theme::PAGE_SPACING)
            .padding(theme::PAGE_PADDING),
    )
    .into()
}

pub fn to_japanese(te: &ToJapanese, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&te.input_text, is_cached));

    // Meanings Layout
    if !te.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &te.meanings {
            let item: Element = if meaning.point.is_empty() {
                list_item(text_block(&meaning.definition).wrap())
            } else {
                comparison(&meaning.point, &meaning.definition)
            };
            meanings.push(item);
        }
        children.push(section(
            "Meanings",
            vstack(meanings).spacing(theme::SECTION_SPACING),
        ));
    }

    // Examples Layout
    if !te.examples.is_empty() {
        let mut examples: Vec<Element> = Vec::new();
        for example in &te.examples {
            examples.push(comparison(&example.zh, &example.ja));
        }
        children.push(section(
            "Examples",
            vstack(examples).spacing(theme::SECTION_SPACING),
        ));
    }

    scroll_viewer(
        vstack(children)
            .spacing(theme::PAGE_SPACING)
            .padding(theme::PAGE_PADDING),
    )
    .into()
}
