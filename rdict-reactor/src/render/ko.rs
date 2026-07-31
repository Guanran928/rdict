use rdict_core::parse::ko::{ToChinese, ToKorean};
use windows_reactor::*;

use crate::components::{comparison, list_item, pos_chip, section, title};
use crate::theme;

pub fn to_chinese(tc: &ToChinese, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&tc.input_text, is_cached));

    // Meanings Layout
    if !tc.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &tc.meanings {
            let mut inner: Vec<Element> = Vec::new();
            if let Some(pos) = &meaning.part_of_speech {
                inner.push(pos_chip(pos));
            }
            for definition in &meaning.definitions {
                inner.push(list_item(text_block(definition).wrap()));
            }
            if let Some(example) = &meaning.example {
                inner.push(
                    text_block(&example.ko)
                        .font_size(14.0)
                        .foreground(ThemeRef::SecondaryText)
                        .wrap()
                        .into(),
                );
                inner.push(
                    text_block(&example.zh)
                        .font_size(14.0)
                        .foreground(ThemeRef::SecondaryText)
                        .wrap()
                        .into(),
                );
            }
            meanings.push(vstack(inner).spacing(4.0).into());
        }
        children.push(section(
            "Meanings",
            vstack(meanings).spacing(theme::SECTION_SPACING),
        ));
    }

    scroll_viewer(
        vstack(children)
            .spacing(theme::PAGE_SPACING)
            .padding(theme::PAGE_PADDING),
    )
    .into()
}

pub fn to_korean(te: &ToKorean, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&te.input_text, is_cached));

    // Meanings Layout
    if !te.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &te.meanings {
            let mut inner: Vec<Element> = Vec::new();
            if let Some(pos) = &meaning.part_of_speech {
                inner.push(pos_chip(pos));
            }
            for definition in &meaning.definitions {
                inner.push(list_item(text_block(definition).wrap()));
            }
            meanings.push(vstack(inner).spacing(4.0).into());
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
            examples.push(comparison(&example.zh, &example.ko));
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
