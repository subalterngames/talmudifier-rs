use crate::font::language::Language;
use serde::{Deserialize, Serialize};

/// The title text on a page.
#[derive(Clone, Deserialize, Serialize)]
pub struct Title {
    pub title: String,
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
