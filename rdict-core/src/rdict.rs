use crate::Error;
use crate::model::{Language, Voice};
use crate::parse::{DictPage, NotFound, selectors};
use crate::parse::{en, fr, ja, ko};
use log::{debug, info};
use reqwest::Client;
use scraper::Html;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePool;
use std::fs;

#[derive(Debug, Clone)]

pub struct Rdict {
    client: Client,
    base_url: String,
    pool: Option<SqlitePool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchedResult {
    pub data: TranslationData,
    pub voices: Vec<Voice>,
    pub is_cached: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum TranslationData {
    #[serde(rename = "from_english")]
    FromEnglish(en::ToChinese),
    #[serde(rename = "to_english")]
    ToEnglish(en::ToEnglish),

    #[serde(rename = "from_french")]
    FromFrench(fr::ToChinese),
    #[serde(rename = "to_french")]
    ToFrench(fr::ToFrench),

    #[serde(rename = "from_korean")]
    FromKorean(ko::ToChinese),
    #[serde(rename = "to_korean")]
    ToKorean(ko::ToKorean),

    #[serde(rename = "from_japanese")]
    FromJapanese(ja::ToChinese),
    #[serde(rename = "to_japanese")]
    ToJapanese(ja::ToJapanese),

    #[serde(rename = "not_found")]
    NotFound(NotFound),
}

impl TranslationData {
    fn as_render(&self) -> &dyn crate::render::Render {
        match self {
            Self::FromEnglish(x) => x,
            Self::ToEnglish(x) => x,
            Self::FromFrench(x) => x,
            Self::ToFrench(x) => x,
            Self::FromKorean(x) => x,
            Self::ToKorean(x) => x,
            Self::FromJapanese(x) => x,
            Self::ToJapanese(x) => x,
            Self::NotFound(x) => x,
        }
    }

    #[must_use]
    pub fn voices(&self) -> Vec<Voice> {
        match self {
            Self::FromEnglish(x) => vec![
                Voice::english("UK", &x.input_text, "1"),
                Voice::english("US", &x.input_text, "2"),
            ],
            Self::ToEnglish(x) => vec![Voice::language("Chinese", &x.input_text, "zh")],
            Self::FromFrench(x) => vec![Voice::language("French", &x.input_text, "fr")],
            Self::ToFrench(x) => vec![Voice::language("Chinese", &x.input_text, "zh")],
            Self::FromKorean(x) => vec![Voice::language("Korean", &x.input_text, "ko")],
            Self::ToKorean(x) => vec![Voice::language("Chinese", &x.input_text, "zh")],
            Self::FromJapanese(x) => vec![Voice::language("Japanese", &x.input_text, "ja")],
            Self::ToJapanese(x) => vec![Voice::language("Chinese", &x.input_text, "zh")],
            Self::NotFound(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn render_colored(&self) -> String {
        self.as_render().render_colored()
    }

    #[must_use]
    pub fn render_plain(&self) -> String {
        self.as_render().render_plain()
    }
}

impl FetchedResult {
    fn new(data: TranslationData, is_cached: bool) -> Self {
        let voices = data.voices();

        Self {
            data,
            voices,
            is_cached,
        }
    }

    #[must_use]
    pub fn render_colored(&self) -> String {
        crate::render::append_voices_colored(self.data.render_colored(), &self.voices)
    }

    #[must_use]
    pub fn render_plain(&self) -> String {
        crate::render::append_voices_plain(self.data.render_plain(), &self.voices)
    }
}

impl Rdict {
    pub async fn new(
        base_url: &str,
        cache_db_path: Option<std::path::PathBuf>,
    ) -> Result<Self, Error> {
        let pool: Option<SqlitePool> = if let Some(db_path) = cache_db_path {
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let db_url = db_path
                .to_str()
                .ok_or_else(|| Error::InvalidDatabasePath(db_path.clone()))?;

            if !sqlx::Sqlite::database_exists(db_url).await? {
                sqlx::Sqlite::create_database(db_url).await?;
            }

            let pool = SqlitePool::connect(db_url).await?;

            sqlx::query(
                "CREATE TABLE IF NOT EXISTS cache (
                    text TEXT NOT NULL,
                    source_lang TEXT NOT NULL,
                    data TEXT NOT NULL,
                    PRIMARY KEY (text, source_lang)
                );",
            )
            .execute(&pool)
            .await?;

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

    pub async fn get_results(
        &self,
        input_text: &str,
        language: Language,
    ) -> Result<FetchedResult, Error> {
        debug!("Getting result. input_text: {input_text}, language: {language:?}");

        if let Some(result) = self.try_cache(input_text, language).await? {
            return Ok(result);
        }

        let data = self.fetch_and_parse(input_text, language).await?;

        if let Some(pool) = &self.pool {
            self.write_cache(pool, input_text, language, &data).await?;
        }

        Ok(FetchedResult::new(data, false))
    }

    async fn try_cache(
        &self,
        input_text: &str,
        language: Language,
    ) -> Result<Option<FetchedResult>, Error> {
        let pool = match &self.pool {
            Some(p) => p,
            None => return Ok(None),
        };

        debug!("Caching enabled, trying fetch data from cache.");

        let row_result = sqlx::query("SELECT data FROM cache WHERE text = ? AND source_lang = ?")
            .bind(input_text)
            .bind(language.code())
            .fetch_optional(pool)
            .await;

        let row = match row_result {
            Ok(Some(row)) => row,
            Ok(None) => {
                info!("Translation cache missed, fetching translation data again.");
                return Ok(None);
            }
            Err(_) => {
                info!(
                    "Database error, deleting row from SQLite cache and fetching translation data again."
                );
                sqlx::query("DELETE FROM cache WHERE text = ? AND source_lang = ?")
                    .bind(input_text)
                    .bind(language.code())
                    .execute(pool)
                    .await?;
                return Ok(None);
            }
        };

        let data_str: String = row.try_get("data")?;

        let data: TranslationData = if let Ok(v) = serde_json::from_str(&data_str) {
            v
        } else {
            sqlx::query("DELETE FROM cache WHERE text = ? AND source_lang = ?")
                .bind(input_text)
                .bind(language.code())
                .execute(pool)
                .await?;
            return Ok(None);
        };

        Ok(Some(FetchedResult::new(data, true)))
    }

    pub async fn fetch_voice(&self, voice: &Voice) -> Result<Vec<u8>, Error> {
        Ok(self
            .client
            .get(&voice.url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec())
    }

    async fn fetch_and_parse(
        &self,
        input_text: &str,
        language: Language,
    ) -> Result<TranslationData, Error> {
        let html = self.fetch_text_html(input_text, language.code()).await?;

        let binding = Html::parse_document(&html);
        let dict_page = DictPage::new(
            binding
                .select(&selectors::BODY)
                .next()
                .ok_or(Error::Parse("no .search_result-dict found".into()))?,
        );

        match dict_page.parse_translation_direction() {
            Err(Error::NoTranslationResults) => {
                Ok(TranslationData::NotFound(dict_page.not_found()?))
            }

            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    async fn write_cache(
        &self,
        pool: &SqlitePool,
        input_text: &str,
        language: Language,
        data: &TranslationData,
    ) -> Result<(), Error> {
        let serialized = serde_json::to_string(&data)?;

        sqlx::query("INSERT OR REPLACE INTO cache (text, source_lang, data) VALUES (?, ?, ?)")
            .bind(input_text)
            .bind(language.code())
            .bind(serialized)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn fetch_text_html(&self, text: &str, lang: &str) -> Result<String, reqwest::Error> {
        let url = format!("{}/result", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("word", text), ("lang", lang)])
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
