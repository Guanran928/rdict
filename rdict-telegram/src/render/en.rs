use crate::render::RenderExt;
use rdict_core::parse::en;
use std::fmt::Write;

impl RenderExt for en::ToChinese {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if self.pronunciation.uk.is_some() || self.pronunciation.us.is_some() {
            writeln!(output, "# Pronunciation").unwrap();

            if let Some(uk) = &self.pronunciation.uk {
                writeln!(output, "英：[{uk}]").unwrap();
            }

            if let Some(us) = &self.pronunciation.us {
                writeln!(output, "美：[{us}]").unwrap();
            }

            writeln!(output).unwrap();
        }

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for me in &self.meanings {
                if let Some(pa) = &me.part_of_speech {
                    writeln!(output, "[{pa}]").unwrap();
                }
                for de in &me.definitions {
                    writeln!(output, "* {de}").unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}

impl RenderExt for en::ToEnglish {
    fn render(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{}", &self.input_text).unwrap();
        writeln!(output).unwrap();

        if !self.meanings.is_empty() {
            writeln!(output, "# Meanings").unwrap();
            for me in &self.meanings {
                writeln!(output, "* {me}").unwrap();
            }
            writeln!(output).unwrap();
        }

        if !self.examples.is_empty() {
            writeln!(output, "# Examples").unwrap();
            for ex in &self.examples {
                writeln!(output, "* {}", ex.en).unwrap();
                writeln!(output, "  {}", ex.zh).unwrap();
            }
            writeln!(output).unwrap();
        }

        output.trim_end().to_string()
    }
}
