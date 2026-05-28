mod en;
mod fr;
mod ja;
mod ko;

use rdict_core::{parse::NotFound, rdict::TranslationData};
use std::fmt::Write;

#[must_use]
pub trait RenderExt {
    fn render(&self) -> String;
}

impl RenderExt for NotFound {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "Did you mean:").unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {suggestion}").unwrap();
        }

        output.trim_end().to_string()
    }
}

pub trait TranslationDataExt {
    fn render(&self) -> String;
}

impl TranslationDataExt for TranslationData {
    fn render(&self) -> String {
        as_render(self).render()
    }
}

fn as_render(data: &TranslationData) -> &dyn RenderExt {
    match data {
        TranslationData::FromEnglish(x) => x,
        TranslationData::ToEnglish(x) => x,
        TranslationData::FromFrench(x) => x,
        TranslationData::ToFrench(x) => x,
        TranslationData::FromKorean(x) => x,
        TranslationData::ToKorean(x) => x,
        TranslationData::FromJapanese(x) => x,
        TranslationData::ToJapanese(x) => x,
        TranslationData::NotFound(x) => x,
    }
}
