use crate::font::language::Language;
use serde::{Deserialize, Serialize};

/// The title text on a page.
#[derive(Clone, Deserialize, Serialize)]
pub struct Title {
    /// The text of the title.
    pub title: String,
    /// The language of the title. Defaults to English.
    #[serde(default)]
    pub language: Language,
}

impl From<&str> for Title {
    fn from(value: &str) -> Self {
        Self {
            title: value.to_string(),
            language: Language::English,
        }
    }
}
