use crate::render::{RenderExt, colors};
use owo_colors::OwoColorize;
use rdict_core::parse::ja;
use std::fmt::Write;

impl RenderExt for ja::ToChinese {
    fn render_colored(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text.bold()).unwrap();
        writeln!(output).unwrap();

        if let Some(pr) = &self.pronunciation {
            writeln!(output, "{}", "# Pronunciation".style(colors::MUTED)).unwrap();
            writeln!(
                output,
                "[{} | {}]",
                pr.kana.style(colors::PRIMARY),
                pr.romaji.style(colors::PRIMARY)
            )
            .unwrap();
            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "{}", "# Meanings".style(colors::MUTED)).unwrap();

            if let Some(pos) = &self.part_of_speech {
                writeln!(output, "[{}]", pos).unwrap();
            }

            for m in &self.meanings {
                writeln!(output, "* {}", m.style(colors::PRIMARY)).unwrap();
            }
            writeln!(output).unwrap();
        }

        if let Some(ex) = &self.exam {
            writeln!(output, "{}", "# Exam".style(colors::MUTED)).unwrap();
            writeln!(output, "{}", ex.style(colors::PRIMARY)).unwrap();
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "{}", "# Examples".style(colors::MUTED)).unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.ja.style(colors::PRIMARY)).unwrap();
                writeln!(output, "  {}", ex.zh.style(colors::SECONDARY)).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }

    fn render_plain(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if let Some(pr) = &self.pronunciation {
            writeln!(output, "# Pronunciation").unwrap();
            writeln!(output, "[{} | {}]", pr.kana, pr.romaji).unwrap();
            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();

            if let Some(pos) = &self.part_of_speech {
                writeln!(output, "{pos}").unwrap();
            }

            for m in &self.meanings {
                writeln!(output, "* {m}").unwrap();
            }
            writeln!(output).unwrap();
        }

        if let Some(ex) = &self.exam {
            writeln!(output, "# Exam").unwrap();
            writeln!(output, "{ex}").unwrap();
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.ja).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}

impl RenderExt for ja::ToJapanese {
    fn render_colored(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text.bold()).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "{}", "# Meanings".style(colors::MUTED)).unwrap();
            for m in &self.meanings {
                if !m.point.is_empty() {
                    write!(output, "[{}] ", m.point.style(colors::PRIMARY)).unwrap();
                }
                writeln!(output, "{}", m.definition.style(colors::PRIMARY)).unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "{}", "# Examples".style(colors::MUTED)).unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.ja.style(colors::PRIMARY)).unwrap();
                writeln!(output, "  {}", ex.zh.style(colors::SECONDARY)).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }

    fn render_plain(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for m in &self.meanings {
                if !m.point.is_empty() {
                    write!(output, "[{point}] ", point = m.point).unwrap();
                }
                writeln!(output, "{def}", def = m.definition).unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.ja).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}
