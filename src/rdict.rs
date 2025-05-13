use crate::parse::{ToChinese, ToEnglish, to_chinese, to_english};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::blocking::Client;
use rustyline::DefaultEditor;
use std::fmt::Write;
use std::time::Duration;

pub struct Rdict {
    client: Client,
    base_url: String,
    conn: Option<rusqlite::Connection>,
}

impl Rdict {
    pub fn new(base_url: &str, conn: Option<rusqlite::Connection>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_owned(),
            conn,
        }
    }

    pub fn output_results(&self, word: &str) -> Result<()> {
        let is_cjk = Self::contains_cjk(word);

        // Retrieve from cache if available
        if let Some(conn) = &self.conn {
            if is_cjk {
                let mut stmt = conn
                    .prepare("SELECT data FROM to_english_results WHERE word = ?1")
                    .context("Failed to prepare SQL statement for Chinese cache lookup")?;
                let mut rows = stmt.query([word])?;
                if let Some(row) = rows.next()? {
                    let data: String = row.get(0).context("Failed to get data from row")?;
                    let result: ToEnglish = serde_json::from_str(&data)
                        .context("Failed to deserialize data to ToEnglish")?;
                    self.output_english(word, &result, false)?;
                    println!("  {}\n", format!("[ {word} ] From cache").bright_black());
                    return Ok(());
                }
            } else {
                let mut stmt = conn
                    .prepare("SELECT data FROM to_chinese_results WHERE word = ?1")
                    .context("Failed to prepare SQL statement for English cache lookup")?;
                let mut rows = stmt.query([word])?;
                if let Some(row) = rows.next()? {
                    let data: String = row.get(0).context("Failed to get data from row")?;
                    let result: ToChinese = serde_json::from_str(&data)
                        .context("Failed to deserialize data to ToChinese")?;
                    self.output_chinese(word, &result, false)?;
                    println!("  {}\n", format!("[ {word} ] From cache").bright_black());
                    return Ok(());
                }
            }
        }

        // If not found in cache, fetch from the web
        let html = self.fetch_word_html(word).context("Error fetching HTML")?;
        if is_cjk {
            let result = to_english(&html)?;
            self.output_english(word, &result, true)?;
        } else {
            let result = to_chinese(&html)?;
            self.output_chinese(word, &result, true)?;
        }

        Ok(())
    }

    pub fn output_english(
        &self,
        word: &str,
        result: &ToEnglish,
        save_to_cache: bool,
    ) -> Result<()> {
        if save_to_cache {
            if let Some(conn) = &self.conn {
                let data = serde_json::to_string(&result).context("Error serializing result")?;
                conn.execute(
                    "INSERT OR REPLACE INTO to_english_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                )
                .context("Error saving to cache")?;
            }
        }

        let mut output = "\n".to_owned();

        if !result.translations.is_empty() {
            writeln!(output, "  {}", "# Translations".bright_black())?;
            for tr in &result.translations {
                writeln!(output, "  * {}", tr.green())?;
            }
            writeln!(output)?;
        }

        if !result.example_sentences.is_empty() {
            writeln!(output, "  {}", "# Examples".bright_black())?;
            for ex in &result.example_sentences {
                writeln!(output, "  * {}", ex.english_sentence.green())?;
                writeln!(output, "    {}", ex.chinese_sentence.magenta())?;
            }
            writeln!(output)?;
        }

        print!("{output}");

        Ok(())
    }

    pub fn output_chinese(
        &self,
        word: &str,
        result: &ToChinese,
        save_to_cache: bool,
    ) -> Result<()> {
        if save_to_cache {
            if let Some(conn) = &self.conn {
                let data = serde_json::to_string(&result).context("Error serializing result")?;
                conn.execute(
                    "INSERT OR REPLACE INTO to_chinese_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                )
                .context("Error saving to cache")?;
            }
        }

        let mut output = "\n".to_owned();

        if !result.phonetic.uk.is_empty() || !result.phonetic.us.is_empty() {
            writeln!(output, "  {}", "# Phonetics".bright_black())?;
            writeln!(output, "  英：[{}]", result.phonetic.uk.green())?;
            writeln!(output, "  美：[{}]", result.phonetic.us.green())?;
            writeln!(output)?;
        }

        if !result.translations.is_empty() {
            writeln!(output, "  {}", "# Translations".bright_black())?;
            for t in &result.translations {
                if !t.english_word_type.is_empty() {
                    writeln!(output, "  [{}]", t.english_word_type)?;
                }
                for tr in &t.chinese_translation {
                    writeln!(output, "  * {}", tr.green())?;
                }
            }
            writeln!(output)?;
        }

        if !result.example_sentences.is_empty() {
            writeln!(output, "  {}", "# Examples".bright_black())?;
            for ex in &result.example_sentences {
                writeln!(output, "  * {}", ex.english_sentence.green())?;
                writeln!(output, "    {}", ex.chinese_sentence.magenta())?;
            }
            writeln!(output)?;
        }

        print!("{output}");
        Ok(())
    }

    pub fn fetch_word_html(&self, word: &str) -> Result<String, reqwest::Error> {
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Fetching data...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );

        let url = format!("{}/result", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("word", word), ("lang", "en")])
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
            )
            .send()?
            .text()?;

        Ok(response)
    }

    pub fn interactive_mode(&self) -> rustyline::Result<()> {
        let mut rl = DefaultEditor::new()?;
        loop {
            let readline = rl.readline(format!("{}# ", "[rdict]".green()).as_str());
            match readline {
                Ok(line) => {
                    if !line.is_empty() {
                        rl.add_history_entry(line.as_str())?;
                        let word = line.as_str().trim();
                        if let Err(err) = Self::output_results(self, word) {
                            println!("Error: {err:?}");
                        }
                    }
                }
                Err(err) => {
                    println!("Error: {err:?}");
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn contains_cjk(word: &str) -> bool {
        word.chars()
            .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use mockito::{Matcher, Server};
    use predicates::prelude::*;

    #[test]
    fn test_contains_cjk_with_cjk() {
        assert!(Rdict::contains_cjk("你好"));
    }

    #[test]
    fn test_contains_cjk_without_cjk() {
        assert!(!Rdict::contains_cjk("hello"));
    }

    #[test]
    fn test_contains_cjk_mixed_input() {
        assert!(Rdict::contains_cjk("hello你好"));
    }

    #[test]
    fn test_contains_cjk_empty() {
        assert!(!Rdict::contains_cjk(""));
    }

    #[test]
    fn test_cmd_stdin_empty() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("rdict")?;
        cmd.write_stdin("")
            .assert()
            .failure()
            .stderr(predicate::str::contains("No word specified"));

        Ok(())
    }

    #[test]
    fn test_fetch_word_html_success_with_mock_server() {
        let mut server = Server::new();

        let mock = server
            .mock("GET", "/result")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("word".into(), "hello".into()),
                Matcher::UrlEncoded("lang".into(), "en".into()),
            ]))
            .with_status(200)
            .with_body(include_str!("fixtures/hello_response.html"))
            .create();

        let rdict = Rdict {
            client: Client::new(),
            base_url: server.url(),
            conn: None,
        };

        let html = rdict.fetch_word_html("hello").unwrap();
        assert!(html.contains("Hello"));
        mock.assert();
    }
}
