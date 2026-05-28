use crate::render::RenderExt;
use rdict_core::parse::fr;
use std::fmt::Write;

impl RenderExt for fr::ToChinese {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if let Some(ph) = &self.pronunciation {
            writeln!(output, "# Pronunciation").unwrap();
            writeln!(output, "[{ph}]").unwrap();
            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for m in &self.meanings {
                writeln!(output, "* {m}").unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.fr).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}

impl RenderExt for fr::ToFrench {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for m in &self.meanings {
                writeln!(output, "* {m}").unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.fr).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}
