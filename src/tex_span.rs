use cosmic_text::{Attrs, Style, Weight};

use crate::{index::Index, tex};

pub enum TexSpan {
    Regular(Vec<String>),
    Bold(Vec<String>),
    Italic(Vec<String>),
    BoldItalic(Vec<String>),
}

impl TexSpan {
    pub fn get_tex(&self, index: Option<&Index>) -> String {
        let words = match self {
            Self::Regular(words)
            | Self::Bold(words)
            | Self::Italic(words)
            | Self::BoldItalic(words) => {
                if words.is_empty() {
                    return String::new();
                } else {
                    match index {
                        Some(index) => words[0..index.word].join(" "),
                        None => words.join(" "),
                    }
                }
            }
        };
        match self {
            Self::Regular(_) => words,
            Self::Bold(_) => tex!("textbf", words),
            Self::Italic(_) => tex!("textit", words),
            Self::BoldItalic(_) => tex!("textbf", tex!("textit", words)),
        }
    }
}

impl From<&(Vec<String>, Attrs<'_>)> for TexSpan {
    fn from(value: &(Vec<String>, Attrs<'_>)) -> Self {
        let words = value.0.clone();
        match (value.1.style, value.1.weight) {
            (Style::Normal, Weight::BOLD) => Self::Bold(words),
            (Style::Italic, Weight::NORMAL) => Self::Italic(words),
            (Style::Italic, Weight::BOLD) => Self::BoldItalic(words),
            _ => Self::Regular(words),
        }
    }
}
