use crate::parse::{Data, ToChinese, ToEnglish, to_chinese, to_english};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::blocking::Client;
use rustyline::DefaultEditor;
use serde::Serialize;
use std::fmt::Write;
use std::time::Duration;

pub struct Rdict {
    client: Client,
    base_url: String,
    conn: Option<rusqlite::Connection>,
    format: Format,
}

#[derive(Debug)]
pub enum Format {
    Pretty,
    Json,
}

pub struct FetchedResult {
    data: Data,
    is_cached: bool,
}

#[derive(Serialize)]
struct OutputWrapper<'a, T> {
    r#type: &'a str,
    data: &'a T,
}

impl Rdict {
    pub fn new(base_url: &str, conn: Option<rusqlite::Connection>, format: Format) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_owned(),
            conn,
            format,
        }
    }

    pub fn output_results(&self, word: &str) -> Result<()> {
        let result = self.get_results(word)?;

        match self.format {
            Format::Json => match &result.data {
                Data::ToChinese(data) => {
                    let wrapper = OutputWrapper {
                        r#type: "ToChinese",
                        data,
                    };
                    println!("{}", serde_json::to_string_pretty(&wrapper)?);
                }
                Data::ToEnglish(data) => {
                    let wrapper = OutputWrapper {
                        r#type: "ToEnglish",
                        data,
                    };
                    println!("{}", serde_json::to_string_pretty(&wrapper)?);
                }
            },
            Format::Pretty => {
                match &result.data {
                    Data::ToChinese(tc) => output_chinese(tc)?,
                    Data::ToEnglish(te) => output_english(te)?,
                }
                if result.is_cached {
                    println!("  {}\n", format!("[ {word} ] From cache").bright_black());
                }
            }
        }

        Ok(())
    }

    pub fn get_results(&self, word: &str) -> Result<FetchedResult> {
        let is_cjk = contains_cjk(word);

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

                    return Ok(FetchedResult {
                        data: Data::ToEnglish(result),
                        is_cached: true,
                    });
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

                    return Ok(FetchedResult {
                        data: Data::ToChinese(result),
                        is_cached: true,
                    });
                }
            }
        }

        // If not found in cache, fetch from the web, and save to cache
        let html = self.fetch_word_html(word).context("Error fetching HTML")?;

        if is_cjk {
            let result = to_english(&html)?;
            if let Some(conn) = &self.conn {
                let data = serde_json::to_string(&result).context("Error serializing result")?;
                conn.execute(
                    "INSERT OR REPLACE INTO to_english_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                )
                .context("Error saving to cache")?;
            }

            Ok(FetchedResult {
                data: Data::ToEnglish(result),
                is_cached: false,
            })
        } else {
            let result = to_chinese(&html)?;
            if let Some(conn) = &self.conn {
                let data = serde_json::to_string(&result).context("Error serializing result")?;
                conn.execute(
                    "INSERT OR REPLACE INTO to_chinese_results (word, data) VALUES (?1, ?2)",
                    rusqlite::params![word, data],
                )
                .context("Error saving to cache")?;
            }

            Ok(FetchedResult {
                data: Data::ToChinese(result),
                is_cached: false,
            })
        }
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
            let readline = if cfg!(target_family = "windows") {
                rl.readline("[rdict]# ")
            } else {
                rl.readline(format!("{}# ", "[rdict]".green()).as_str())
            };
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
}

fn contains_cjk(word: &str) -> bool {
    word.chars()
        .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch))
}

fn output_chinese(result: &ToChinese) -> Result<()> {
    let mut output = "\n".to_owned();

    if result.phonetic.uk.is_some() || result.phonetic.us.is_some() {
        writeln!(output, "  {}", "# Phonetics".bright_black())?;

        if let Some(ref uk) = result.phonetic.uk {
            writeln!(output, "  英：[{}]", uk.green())?;
        }

        if let Some(ref us) = result.phonetic.us {
            writeln!(output, "  美：[{}]", us.green())?;
        }

        writeln!(output)?;
    }

    if !result.translations.is_empty() {
        writeln!(output, "  {}", "# Translations".bright_black())?;
        for t in &result.translations {
            if let Some(ref ty) = t.english_word_type {
                writeln!(output, "  [{ty}]")?;
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

fn output_english(result: &ToEnglish) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use mockito::{Matcher, Server};
    use predicates::prelude::*;

    #[test]
    fn test_contains_cjk_with_cjk() {
        assert!(contains_cjk("你好"));
    }

    #[test]
    fn test_contains_cjk_without_cjk() {
        assert!(!contains_cjk("hello"));
    }

    #[test]
    fn test_contains_cjk_mixed_input() {
        assert!(contains_cjk("hello你好"));
    }

    #[test]
    fn test_contains_cjk_empty() {
        assert!(!contains_cjk(""));
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
            format: self::Format::Pretty,
        };

        let html = rdict.fetch_word_html("hello").unwrap();
        assert!(html.contains("Hello"));
        mock.assert();
    }
}
