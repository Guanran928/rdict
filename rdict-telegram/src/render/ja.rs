use crate::render::RenderExt;
use rdict_core::parse::ja;
use std::fmt::Write;

impl RenderExt for ja::ToChinese {
    fn render(&self) -> String {
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
    fn render(&self) -> String {
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
