use anyhow::{Result, ensure};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Phonetic {
    pub uk: Option<String>,
    pub us: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExampleSentence {
    pub english_sentence: String,
    pub chinese_sentence: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToChineseTranslation {
    pub english_word_type: Option<String>,
    pub chinese_translation: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToChinese {
    pub phonetic: Phonetic,
    pub translations: Vec<ToChineseTranslation>,
    pub example_sentences: Vec<ExampleSentence>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ToEnglish {
    pub translations: Vec<String>,
    pub example_sentences: Vec<ExampleSentence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Data {
    ToChinese(ToChinese),
    ToEnglish(ToEnglish),
}

/// Parses English, returns Chinese
pub fn to_chinese(html: &str) -> Result<ToChinese> {
    let document = Html::parse_document(html);
    let mut result = ToChinese::default();

    // Phonetic
    let per_phone_selector = Selector::parse(".phone_con .per-phone").unwrap();
    let phonetic_selector = Selector::parse(".phonetic").unwrap();

    for (i, element) in document.select(&per_phone_selector).enumerate() {
        let text = element
            .select(&phonetic_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned());

        match i {
            0 => {
                result.phonetic.uk = text
                    .as_deref()
                    .map(|s| s.trim_matches('/').trim())
                    .filter(|s| !s.is_empty())
                    .map(std::string::ToString::to_string);
            }
            1 => {
                result.phonetic.us = text
                    .as_deref()
                    .map(|s| s.trim_matches('/').trim())
                    .filter(|s| !s.is_empty())
                    .map(std::string::ToString::to_string);
            }
            _ => unreachable!(),
        }
    }

    // Translations
    let word_exp_selector = Selector::parse(".trans-container .basic .word-exp").unwrap();
    let trans_selector = Selector::parse(".trans").unwrap();
    let pos_selector = Selector::parse(".pos").unwrap();

    for element in document.select(&word_exp_selector) {
        let chinese_translation: Vec<String> = element
            .select(&trans_selector)
            .next()
            .map(|e| {
                e.text()
                    .collect::<String>()
                    .trim()
                    .split('；')
                    .map(|s| s.trim().to_owned())
                    .collect()
            })
            .unwrap_or_default();

        let english_word_type = element
            .select(&pos_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned());

        if !chinese_translation.is_empty() || english_word_type.is_some() {
            result.translations.push(ToChineseTranslation {
                english_word_type,
                chinese_translation,
            });
        }
    }

    // Example sentences
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let english_sentence = element
            .select(&sen_eng_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let chinese_sentence = element
            .select(&sen_ch_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !english_sentence.is_empty() || !chinese_sentence.is_empty() {
            result.example_sentences.push(ExampleSentence {
                english_sentence,
                chinese_sentence,
            });
        }
    }

    ensure!(
        !result.translations.is_empty(),
        "No translation results found for English to Chinese"
    );

    Ok(result)
}

/// Parses Chinese, returns English
pub fn to_english(html: &str) -> Result<ToEnglish> {
    let document = Html::parse_document(html);
    let mut result = ToEnglish::default();

    // Translations
    let translation_selector = Selector::parse(".trans-container .basic .col2 .point").unwrap();
    for element in document.select(&translation_selector) {
        let text = element.text().collect::<String>().trim().to_owned();
        if !text.is_empty() {
            result.translations.push(text);
        }
    }

    // Example sentences
    let example_selector = Selector::parse(".trans-container .mcols-layout .col2").unwrap();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();

    for element in document.select(&example_selector) {
        let english_sentence = element
            .select(&sen_eng_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let chinese_sentence = element
            .select(&sen_ch_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if !english_sentence.is_empty() || !chinese_sentence.is_empty() {
            result.example_sentences.push(ExampleSentence {
                english_sentence,
                chinese_sentence,
            });
        }
    }

    ensure!(
        !result.translations.is_empty(),
        "No translation results found for Chinese to English"
    );

    Ok(result)
}
