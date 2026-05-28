use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ja")]
    Japanese,
}

impl Language {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
            Self::Korean => "ko",
            Self::Japanese => "ja",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::English => write!(f, "English"),
            Self::French => write!(f, "French"),
            Self::Korean => write!(f, "Korean"),
            Self::Japanese => write!(f, "Japanese"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Voice {
    pub label: String,
    pub url: String,
}

impl Voice {
    #[must_use]
    pub fn english(label: &str, text: &str, kind: &str) -> Self {
        Self::youdao(label, text, &[("type", kind)])
    }

    #[must_use]
    pub fn language(label: &str, text: &str, le: &str) -> Self {
        Self::youdao(label, text, &[("le", le)])
    }

    fn youdao(label: &str, text: &str, params: &[(&str, &str)]) -> Self {
        let mut url =
            reqwest::Url::parse("https://dict.youdao.com/dictvoice").expect("valid voice URL");

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("audio", text);
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }

        Self {
            label: label.to_owned(),
            url: url.to_string(),
        }
    }
}
