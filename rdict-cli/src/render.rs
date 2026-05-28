mod en;
mod fr;
mod ja;
mod ko;

use owo_colors::OwoColorize;
use rdict_core::{parse::NotFound, rdict::TranslationData};
use std::fmt::Write;

pub mod colors {
    use owo_colors::Style;

    pub const PRIMARY: Style = Style::new().green();

    pub const SECONDARY: Style = Style::new().magenta();

    pub const MUTED: Style = Style::new().bright_black();
}

#[must_use]
pub trait RenderExt {
    // TODO: It's kind of duplicate
    fn render_colored(&self) -> String;
    fn render_plain(&self) -> String;
}

impl RenderExt for NotFound {
    fn render_colored(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", "Did you mean:".style(colors::MUTED)).unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {}", suggestion.style(colors::PRIMARY)).unwrap();
        }

        output.trim_end().to_string()
    }

    fn render_plain(&self) -> String {
        let mut output = String::new();

        writeln!(output, "Did you mean:").unwrap();
        for suggestion in &self.suggestions {
            writeln!(output, "* {suggestion}").unwrap();
        }

        output.trim_end().to_string()
    }
}

pub trait TranslationDataExt {
    fn render_colored(&self) -> String;
    fn render_plain(&self) -> String;
}

impl TranslationDataExt for TranslationData {
    fn render_colored(&self) -> String {
        as_render(self).render_colored()
    }

    fn render_plain(&self) -> String {
        as_render(self).render_plain()
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
