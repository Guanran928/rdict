use crate::parse::{ToChinese, ToEnglish, TranslationData, to_chinese, to_english};
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use reqwest::Client;
use sqlx::Row;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePool;
use std::fmt::Write;
use std::fs;

pub struct Rdict {
    client: Client,
    base_url: String,
    pool: Option<SqlitePool>,
}

#[derive(Debug)]
pub enum Format {
    /// Markdown with ANSI color escape sequences
    MarkdownColored,
    /// Plain Markdown
    Markdown,
    /// Formatted JSON
    Json,
}

pub struct FetchedResult {
    pub data: TranslationData,
    pub is_cached: bool,
}

impl Rdict {
    pub async fn new(base_url: &str, cache_db_path: Option<std::path::PathBuf>) -> Result<Self> {
        let pool: Option<SqlitePool> = if cache_db_path.is_some() {
            let db_path = cache_db_path.unwrap();
            let should_init_db = !db_path.exists();

            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create cache directory at {:?}", parent.display())
                })?;
            }

            let db_url = db_path.to_str().with_context(|| {
                format!("Failed to convert path to string: {}", db_path.display())
            })?;

            if !sqlx::Sqlite::database_exists(db_url).await? {
                sqlx::Sqlite::create_database(db_url).await?;
            }

            let pool = SqlitePool::connect(db_url)
                .await
                .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

            if should_init_db {
                let init_statements = [
                    "CREATE TABLE to_english_results (
                    word TEXT PRIMARY KEY,
                    data TEXT NOT NULL
                );",
                    "CREATE TABLE to_chinese_results (
                    word TEXT PRIMARY KEY,
                    data TEXT NOT NULL
                );",
                ];

                for statement in init_statements {
                    sqlx::query(statement)
                        .execute(&pool)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to initialize SQLite cache database: {}",
                                db_path.display()
                            )
                        })?;
                }
            }

            Some(pool)
        } else {
            None
        };

        Ok(Self {
            client: Client::new(),
            base_url: base_url.to_owned(),
            pool,
        })
    }

    pub async fn get_results(&self, word: &str) -> Result<FetchedResult> {
        let word = word.to_string();
        let is_cjk = contains_cjk(&word);

        // Try cache first
        if let Some(pool) = &self.pool {
            let (table, variant): (&str, fn(String) -> TranslationData) = if is_cjk {
                ("to_english_results", |v| {
                    TranslationData::ToEnglish(serde_json::from_str(&v).unwrap())
                })
            } else {
                ("to_chinese_results", |v| {
                    TranslationData::ToChinese(serde_json::from_str(&v).unwrap())
                })
            };
            let word_for_query = word.clone();
            let query = format!("SELECT data FROM {table} WHERE word = ?");
            let result = sqlx::query(&query)
                .bind(&word_for_query)
                .fetch_optional(pool)
                .await
                .with_context(|| {
                    format!("Failed to look up cached translation for '{word}' in {table}")
                })?;

            if let Some(row) = result {
                let data_str: String = row.try_get("data")?;
                let data = variant(data_str);
                return Ok(FetchedResult {
                    data,
                    is_cached: true,
                });
            }
        }

        // Fetch from web
        let html = self
            .fetch_word_html(&word)
            .await
            .context("Error fetching HTML")?;

        if is_cjk {
            let result = to_english(&html)?;
            if let Some(pool) = &self.pool {
                let data = serde_json::to_string(&result).context("Error serializing result")?;

                sqlx::query("INSERT OR REPLACE INTO to_english_results (word, data) VALUES (?, ?)")
                    .bind(word)
                    .bind(&data)
                    .execute(pool)
                    .await
                    .context("Error saving to English cache")?;
            }

            Ok(FetchedResult {
                data: TranslationData::ToEnglish(result),
                is_cached: false,
            })
        } else {
            let result = to_chinese(&html)?;
            if let Some(pool) = &self.pool {
                let data = serde_json::to_string(&result).context("Error serializing result")?;

                sqlx::query("INSERT OR REPLACE INTO to_chinese_results (word, data) VALUES (?, ?)")
                    .bind(word)
                    .bind(&data)
                    .execute(pool)
                    .await
                    .context("Error saving to Chinese cache")?;
            }

            Ok(FetchedResult {
                data: TranslationData::ToChinese(result),
                is_cached: false,
            })
        }
    }

    async fn fetch_word_html(&self, word: &str) -> Result<String, reqwest::Error> {
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
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }
}

fn contains_cjk(word: &str) -> bool {
    word.chars()
        .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch))
}

pub fn render_chinese_colored(result: &ToChinese) -> Result<String> {
    let mut output = String::new();

    if result.pronunciation.uk.is_some() || result.pronunciation.us.is_some() {
        writeln!(output, "{}", "# Pronunciation".bright_black())?;

        if let Some(ref uk) = result.pronunciation.uk {
            writeln!(output, "英：[{}]", uk.green())?;
        }

        if let Some(ref us) = result.pronunciation.us {
            writeln!(output, "美：[{}]", us.green())?;
        }

        writeln!(output)?;
    }

    if !result.meanings.is_empty() {
        writeln!(output, "{}", "# Meanings".bright_black())?;
        for me in &result.meanings {
            if let Some(ref pa) = me.part_of_speech {
                writeln!(output, "[{pa}]")?;
            }
            for de in &me.definitions {
                writeln!(output, "* {}", de.green())?;
            }
            writeln!(output)?;
        }
    }

    if !result.examples.is_empty() {
        writeln!(output, "{}", "# Examples".bright_black())?;
        for ex in &result.examples {
            writeln!(output, "* {}", ex.en.green())?;
            writeln!(output, "  {}", ex.zh.magenta())?;
        }
        writeln!(output)?;
    }

    Ok(output.trim_end().to_string())
}

pub fn render_english_colored(result: &ToEnglish) -> Result<String> {
    let mut output = String::new();

    if !result.meanings.is_empty() {
        writeln!(output, "{}", "# Meanings".bright_black())?;
        for me in &result.meanings {
            writeln!(output, "* {}", me.green())?;
        }
        writeln!(output)?;
    }

    if !result.examples.is_empty() {
        writeln!(output, "{}", "# Examples".bright_black())?;
        for ex in &result.examples {
            writeln!(output, "* {}", ex.en.green())?;
            writeln!(output, "  {}", ex.zh.magenta())?;
        }
        writeln!(output)?;
    }

    Ok(output.trim_end().to_string())
}

pub fn render_chinese_plain(result: &ToChinese) -> Result<String> {
    let mut output = String::new();

    if result.pronunciation.uk.is_some() || result.pronunciation.us.is_some() {
        writeln!(output, "# Pronunciation")?;

        if let Some(ref uk) = result.pronunciation.uk {
            writeln!(output, "英：[{uk}]")?;
        }

        if let Some(ref us) = result.pronunciation.us {
            writeln!(output, "美：[{us}]")?;
        }

        writeln!(output)?;
    }

    if !result.meanings.is_empty() {
        writeln!(output, "# Meanings")?;
        for me in &result.meanings {
            if let Some(ref pa) = me.part_of_speech {
                writeln!(output, "[{pa}]")?;
            }
            for de in &me.definitions {
                writeln!(output, "* {de}")?;
            }
            writeln!(output)?;
        }
    }

    if !result.examples.is_empty() {
        writeln!(output, "# Examples")?;
        for ex in &result.examples {
            writeln!(output, "* {}", ex.en)?;
            writeln!(output, "  {}", ex.zh)?;
        }
        writeln!(output)?;
    }

    Ok(output.trim_end().to_string())
}

pub fn render_english_plain(result: &ToEnglish) -> Result<String> {
    let mut output = String::new();

    if !result.meanings.is_empty() {
        writeln!(output, "# Meanings")?;
        for me in &result.meanings {
            writeln!(output, "* {me}")?;
        }
        writeln!(output)?;
    }

    if !result.examples.is_empty() {
        writeln!(output, "# Examples")?;
        for ex in &result.examples {
            writeln!(output, "* {}", ex.en)?;
            writeln!(output, "  {}", ex.zh)?;
        }
        writeln!(output)?;
    }

    Ok(output.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};

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

    #[tokio::test]
    async fn test_fetch_word_html_success_with_mock_server() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/result")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("word".into(), "hello".into()),
                Matcher::UrlEncoded("lang".into(), "en".into()),
            ]))
            .with_status(200)
            .with_body(include_str!("fixtures/hello_response.html"))
            .create();

        let client = Rdict::new(&server.url(), None).await.unwrap();

        let html = client.fetch_word_html("hello").await.unwrap();
        assert!(html.contains("Hello"));
        mock.assert();
    }
}
