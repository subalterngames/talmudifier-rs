use cosmic_text::AttrsOwned;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    font::cosmic_font::CosmicFont,
    span::{position::Position, style::Style, Span},
};

lazy_static! {
    static ref RE_ENDS_WITH_COMMAND: Regex = Regex::new(r#"\\(\w+)$"#).unwrap();
    static ref RE_PUNCTUATION: Regex = Regex::new(r#"^(!|;|:|,|\.)"#).unwrap();
    static ref RE_QUOTES: Regex = Regex::new(r#"([“|"](.*?)[”|"])"#).unwrap();
    static ref RE_SPECIAL_CHARS: Regex = Regex::new(r#"(#|\$|%|&|_)"#).unwrap();
}

/// A column of text that can be typeset.
/// Columns try to fill a target number of lines with words.
/// Cosmic is used to get an initial guess at the number of words.
/// Then, Tectonic is used to fill the column.
///
/// `SpanColumn` has a `start` index that are continuously re-sliced for typesetting.
pub struct SpanColumn {
    /// All of the words in the column.
    pub span: Span,
    /// The start index of the `words` slice.
    pub start: usize,
    /// The font used in Cosmic.
    pub cosmic_font: CosmicFont,
    /// The command to set the TeX font.
    pub tex_font: String,
}

impl SpanColumn {
    pub fn new(span: Span, cosmic_font: CosmicFont, tex_font: &str) -> Self {
        Self {
            span,
            start: 0,
            cosmic_font,
            tex_font: tex_font.to_string(),
        }
    }

    pub fn is_word_in_body(&self, index: usize) -> bool {
        self.span.0[index].position == Position::Body
    }

    /// Convert a slice of words into Cosmic text spans.
    pub fn to_cosmic(&self, end: usize) -> Vec<(String, AttrsOwned)> {
        // Strings of words and their associated attributes.
        let mut cosmic_spans = vec![];

        // The current span that we're building.
        let mut span = vec![];

        // The current style.
        let mut style = Style::default();

        // The current Cosmic formatting attributes.
        let mut attrs = self.cosmic_font.regular.clone();

        // Iterate through the words. Ignore citations.
        for word in self.span.0[self.start..end]
            .iter()
            .filter(|w| w.position == Position::Body)
        {
            // Add the word to the current span.
            if style == word.style {
                span.push(word.word.clone());
            }
            // The style changed. Finish the span and set a new style.
            else {
                cosmic_spans.push((span.join(" "), attrs));
                // Reset the span.
                span.clear();
                // Set the new attrs.
                attrs = word.style.attrs(&self.cosmic_font);
                // Remember the style.
                style = word.style;
            }
        }

        // Push the last span.
        if !span.is_empty() {
            cosmic_spans.push((span.join(" "), attrs));
        }
        cosmic_spans
    }

    /// Convert a slice of words to a TeX string.
    pub fn to_tex(&self, end: Option<usize>, marginalia: bool) -> String {
        // Get the end index. If `end` was none, use all remaining words.
        let end = match end {
            Some(end) => end,
            None => self.span.0.len(),
        };

        // Build a column.
        let mut text = self.tex_font.to_string();
        let mut style = Style::default();
        let mut position = Position::default();
        for word in self.span.0[self.start..end].iter() {
            let mut prefixes = vec![];
            let mut suffixes = vec![];
            // We changed the style.
            if style != word.style {
                let (prefix, suffix) = style.get_command(&word.style);
                if let Some(prefix) = prefix {
                    prefixes.push(prefix);
                }
                // Add a suffix to the previous word.
                if let Some(suffix) = suffix {
                    suffixes.push(suffix);
                }
                style = word.style;
            }
            // Change the position.
            if marginalia {
                if position != word.position {
                    let command = position.get_command(&word.position);
                    if let Some(prefix) = command.0 {
                        prefixes.push(prefix);
                    }
                    // Add a suffix to the previous word.
                    if let Some(suffix) = command.1 {
                        suffixes.push(suffix);
                    }

                    position = word.position;
                }
            }
            // Ignore marginalia.
            else if word.position == Position::Margin {
                continue;
            }

            // Add the suffixes.
            suffixes.iter().for_each(|s| text.push_str(s));

            // Add a space if:
            // 1. The last word isn't a command ending with {, for example: \fontcommand
            // 2. The word isn't punctuation.
            if RE_ENDS_WITH_COMMAND.captures(&text).is_some()
                || RE_PUNCTUATION.captures(&word.word).is_none()
            {
                text.push(' ');
            }

            // Add the prefixes.
            prefixes.iter().for_each(|p| text.push_str(p));
            // Add the word.
            text.push_str(&word.word);
        }

        // Close off the styles and citations.
        match style {
            Style::Regular => (),
            Style::Bold | Style::Italic => text.push('}'),
            Style::BoldItalic => text.push_str("}}"),
        }
        if let Position::Margin = position {
            text.push('}');
        }
        Self::santitize_tex(&mut text);
        text
    }

    pub fn done(&self) -> bool {
        self.start >= self.span.0.len()
    }

    /// Sanitize a TeX string.
    fn santitize_tex(tex: &mut String) {
        *tex = RE_SPECIAL_CHARS
            .replace_all(&RE_QUOTES.replace_all(tex, "``$2''"), "\\$1")
            .replace("~", "$\\sim$")
            .replace("<", "\\textless")
            .replace(">", "\\textgreater");
    }
}

#[cfg(test)]
mod tests {
    use crate::{font::cosmic_font::CosmicFont, span::Span, table::span_column::SpanColumn};

    #[test]
    fn test_textit() {
        let md = "*This is italic* and this is regular.";
        let column = get_column(md);
        let tex = column.to_tex(None, true);
        assert_eq!(tex, "\\font \\textit{This is italic} and this is regular.")
    }

    #[test]
    fn test_bold_italic() {
        let md = "**bold** *italic* ***bold and italic*** **bold**";
        let column = get_column(md);
        let tex = column.to_tex(None, true);
        assert_eq!(
            tex,
            "\\font \\textbf{bold} \\textit{italic \\textbf{bold and italic}} \\textbf{bold}"
        )
    }

    #[test]
    fn test_marginnote() {
        let md = "A `footnote *here* and` *there*";
        let column = get_column(md);
        let tex = column.to_tex(None, true);
        assert_eq!(
        tex,
        "\\font A \\marginnote{\\noindent\\justifying\\tiny footnote \\textit{here} and} \\textit{there}"
    );
        let tex = column.to_tex(None, false);
        assert_eq!(tex, "\\font A \\textit{there}");
    }

    fn get_column(md: &str) -> SpanColumn {
        SpanColumn::new(
            Span::from_md(md).unwrap(),
            CosmicFont::default_left(),
            "\\font",
        )
    }
}
