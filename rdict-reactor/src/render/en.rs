use rdict_core::parse::en::{ToChinese, ToEnglish};
use windows_reactor::*;

use crate::components::{comparison, list_item, pos_chip, section, title};
use crate::theme;

pub fn to_chinese(tc: &ToChinese, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&tc.input_text, is_cached));

    // Pronunciation Layout
    let mut accents: Vec<Element> = Vec::new();
    if let Some(uk) = &tc.pronunciation.uk {
        accents.push(
            hstack((
                text_block("英").bold(),
                text_block(format!("[{uk}]")).foreground(ThemeRef::SecondaryText),
            ))
            .spacing(4.0)
            .into(),
        );
    }
    if let Some(us) = &tc.pronunciation.us {
        accents.push(
            hstack((
                text_block("美").bold(),
                text_block(format!("[{us}]")).foreground(ThemeRef::SecondaryText),
            ))
            .spacing(4.0)
            .into(),
        );
    }
    if !accents.is_empty() {
        children.push(hstack(accents).spacing(15.0).into());
    }

    // Meanings Layout
    if !tc.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &tc.meanings {
            let mut definitions_col: Vec<Element> = Vec::new();
            if let Some(p) = &meaning.part_of_speech {
                definitions_col.push(pos_chip(p));
            }
            for definition in &meaning.definitions {
                definitions_col.push(list_item(text_block(definition).wrap()));
            }
            meanings.push(vstack(definitions_col).spacing(2.0).into());
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
            examples.push(comparison(&example.en, &example.zh));
        }
        children.push(section(
            "Examples",
            vstack(examples).spacing(theme::SECTION_SPACING),
        ));
    }

    // Exams Layout
    if !tc.exams.is_empty() {
        let mut exams_row: Vec<Element> = Vec::new();
        for (i, exam) in tc.exams.iter().enumerate() {
            if i != 0 {
                exams_row.push(
                    border(Element::Empty)
                        .background(ThemeRef::DividerStroke)
                        .width(1.0)
                        .height(14.0)
                        .vertical_alignment(VerticalAlignment::Center)
                        .into(),
                );
            }
            exams_row.push(text_block(exam).into());
        }
        children.push(section("Exams", hstack(exams_row).spacing(2.0)));
    }

    scroll_viewer(
        vstack(children)
            .spacing(theme::PAGE_SPACING)
            .padding(theme::PAGE_PADDING),
    )
    .into()
}

pub fn to_english(te: &ToEnglish, is_cached: bool) -> Element {
    let mut children: Vec<Element> = Vec::new();
    children.push(title(&te.input_text, is_cached));

    // Meanings Layout
    if !te.meanings.is_empty() {
        let mut meanings: Vec<Element> = Vec::new();
        for meaning in &te.meanings {
            meanings.push(list_item(text_block(meaning).wrap()));
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
            examples.push(comparison(&example.zh, &example.en));
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
