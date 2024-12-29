use cosmic_text::{Attrs, Style, Weight};

pub enum TexSpan {
    Regular(Vec<String>),
    Bold(Vec<String>),
    Italic(Vec<String>),
    BoldItalic(Vec<String>),
}

impl From<&(String, Attrs<'_>)> for TexSpan {
    fn from(value: &(String, Attrs<'_>)) -> Self {
        let words = value
            .0
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        match (value.1.style, value.1.weight) {
            (Style::Normal, Weight::BOLD) => Self::Bold(words),
            (Style::Italic, Weight::NORMAL) => Self::Italic(words),
            (Style::Italic, Weight::BOLD) => Self::BoldItalic(words),
            _ => Self::Regular(words),
        }
    }
}
