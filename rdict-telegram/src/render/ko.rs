use crate::render::RenderExt;
use rdict_core::parse::ko;
use std::fmt::Write;

impl RenderExt for ko::ToChinese {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for m in &self.meanings {
                if let Some(pos) = &m.part_of_speech {
                    writeln!(output, "[{pos}]").unwrap();
                }
                for de in &m.definitions {
                    writeln!(output, "* {de}").unwrap();
                }
                if let Some(ex) = &m.example {
                    writeln!(output, "  {}", ex.ko).unwrap();
                    writeln!(output, "  {}", ex.zh).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        output.trim_end().to_string()
    }
}

impl RenderExt for ko::ToKorean {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for m in &self.meanings {
                if let Some(pos) = &m.part_of_speech {
                    writeln!(output, "[{pos}]").unwrap();
                }
                for de in &m.definitions {
                    writeln!(output, "* {de}").unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.ko).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}
