use cosmic_text::{Attrs, AttrsOwned, Weight};
use markdown::{mdast::Node, message::Message, to_mdast, Constructs, ParseOptions};

use crate::{font::cosmic_font::CosmicFont, tex};

use position::Position;
use style::Style;

mod latex_command;
mod position;
mod style;

/// A word and its style.
pub struct Word {
    /// A single word.
    pub word: String,
    /// The font style.
    pub style: Style,
    /// The position on
    pub position: Position,
}

impl Word {
    /// Parse raw markdown text and get a vec of words.
    pub fn from_md(md: &str) -> Result<Vec<Self>, Message> {
        let parse_options = ParseOptions {
            constructs: Constructs::gfm(),
            ..Default::default()
        };
        let node = to_mdast(md, &parse_options)?;
        let mut words = vec![];
        // Add the words as nodes.
        Self::add_node(&node, &mut words, Style::default(), Position::default());
        Ok(words)
    }

    /// Convert a slice of words into Cosmic text spans.
    pub fn to_cosmic(words: &[Self], font: &CosmicFont) -> Vec<(String, AttrsOwned)> {
        let mut cosmic_spans = vec![];
        let mut span = vec![];
        let mut style = Style::default();
        let mut attrs = font.regular.clone();
        for word in words.iter() {
            // Add the word to the current span.
            if style == word.style {
                span.push(word.word.clone());
            }
            // Finish the span and set a new style.
            else {
                cosmic_spans.push((span.join(" "), attrs));
                // Reset the span.
                span.clear();
                // Set the new attrs.
                attrs = Self::get_attrs(font, &word.style);
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

    fn get_attrs(font: &CosmicFont, style: &Style) -> AttrsOwned {
        match style {
            Style::Regular => &font.regular,
            Style::Italic => &font.italic,
            Style::Bold => &font.bold,
            Style::BoldItalic => &font.bold_italic
        }.clone()
    }

    pub fn to_tex(words: &[Self], font_command: &str) -> (String, bool) {
        // Try to build a title.
        let title = words
            .iter()
            .filter(|w| w.position == Position::Title)
            .collect::<Vec<&Word>>();
        // Build a column.
        if title.is_empty() {
            let mut text = font_command.to_string();
            let mut style = Style::default();
            let mut position = Position::default();
            for word in words.iter().filter(|w| w.position != Position::Title) {
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
                if position != word.position {
                    let command = position.get_command(&word.position);
                    if let Some(prefix) = command.prefix {
                        prefixes.push(prefix);
                    }
                    // Add a suffix to the previous word.
                    if let Some(suffix) = command.suffix {
                        suffixes.push(suffix);
                    }

                    position = word.position;
                }
                // Add the suffixes.
                suffixes.iter().for_each(|s| text.push_str(*s));
                // Add a space.
                text.push(' ');
                // Add the prefixes.
                prefixes.iter().for_each(|p| text.push_str(*p));
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
            (text, false)
        }
        // Build a title.
        else {
            let title = title
                .iter()
                .map(|w| w.word.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            (tex!("chapter", tex!("daftitle", title)), true)
        }
    }

    /// A words from a node.
    fn add_node(node: &Node, words: &mut Vec<Self>, style: Style, position: Position) {
        match node {
            Node::Root(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, position)),
            Node::InlineCode(node) => Self::add_words(&node.value, words, style, Position::Margin),
            Node::Emphasis(node) => {
                // Add an italic style.
                let style = match style {
                    Style::Bold => Style::BoldItalic,
                    _ => Style::Italic,
                };
                node.children
                    .iter()
                    .for_each(|child| Self::add_node(child, words, style, position));
            }
            Node::Strong(node) => {
                // Add a bold style.
                let style = match style {
                    Style::Italic => Style::BoldItalic,
                    _ => Style::Bold,
                };
                node.children
                    .iter()
                    .for_each(|child| Self::add_node(child, words, style, position));
            }
            Node::Text(text) => Self::add_words(&text.value, words, style, position),
            Node::Heading(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, Position::Title)),
            Node::Paragraph(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, position)),
            _ => (),
        }
    }

    /// Split a string into words and add them to `words`.
    fn add_words(value: &str, words: &mut Vec<Self>, style: Style, position: Position) {
        value.split(' ').filter(|s| !s.is_empty()).for_each(|w| {
            words.push(Word {
                word: w.to_string(),
                style,
                position,
            })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{Position, Style, Word};

    #[test]
    fn test_words() {
        let md = "Regular *italic* **bold and *italic*** and **this**";
        let words = Word::from_md(md).unwrap();
        assert_eq!(&words[0].word, "Regular");
        assert_eq!(words[0].style, Style::Regular);

        assert_eq!(&words[1].word, "italic");
        assert_eq!(words[1].style, Style::Italic);

        assert_eq!(&words[2].word, "bold");
        assert_eq!(words[2].style, Style::Bold);

        assert_eq!(&words[3].word, "and");
        assert_eq!(words[3].style, Style::Bold);

        assert_eq!(&words[4].word, "italic");
        assert_eq!(words[4].style, Style::BoldItalic);

        assert_eq!(&words[5].word, "and");
        assert_eq!(words[5].style, Style::Regular);

        assert_eq!(&words[6].word, "this");
        assert_eq!(words[6].style, Style::Bold);

        for word in words.iter() {
            assert_eq!(word.position, Position::Body);
        }
    }

    #[test]
    fn test_footnote() {
        let md = "A `footnote` *here*";
        let words = Word::from_md(md).unwrap();
        assert_eq!(&words[0].word, "A");
        assert_eq!(words[0].style, Style::Regular);
        assert_eq!(words[0].position, Position::Body);
        assert_eq!(&words[1].word, "footnote");
        assert_eq!(words[1].position, Position::Margin);
        assert_eq!(words[1].style, Style::Regular);
        assert_eq!(words[2].position, Position::Body);
        assert_eq!(words[2].style, Style::Italic);
    }

    #[test]
    fn test_textit() {
        let md = "*This is italic* and this is regular.";
        let words = Word::from_md(md).unwrap();
        let (tex, title) = Word::to_tex(&words, "\\font");
        assert!(!title);
        assert_eq!(tex, "\\font \\textit{This is italic} and this is regular.")
    }

    #[test]
    fn test_bold_italic() {
        let md = "**bold** *italic* ***bold and italic*** **bold**";
        let words = Word::from_md(md).unwrap();
        let (tex, _) = Word::to_tex(&words, "\\font");
        assert_eq!(
            tex,
            "\\font \\textbf{bold} \\textit{italic \\textbf{bold and italic}} \\textbf{bold}"
        )
    }

    #[test]
    fn test_marginnote() {
        let md = "A `footnote` *here*";
        let words = Word::from_md(md).unwrap();
        let (tex, _) = Word::to_tex(&words, "\\font");
        assert_eq!(
            tex,
            "\\font A \\\\marginnote{\\\\noindent\\\\justifying\\\\tiny footnote} \\textit{here}"
        );
    }

    #[test]
    fn test_title() {
        let md = "# This is a title\n\nand this is not.";
        let words = Word::from_md(md).unwrap();
        for word in &words[0..4] {
            assert_eq!(word.style, Style::Regular);
            assert_eq!(word.position, Position::Title);
        }
        for word in &words[4..] {
            assert_eq!(word.style, Style::Regular);
            assert_eq!(word.position, Position::Body);
        }
        let words = Word::from_md(md).unwrap();
        let (tex, title) = Word::to_tex(&words, "\\font");
        assert!(title);
        assert_eq!(tex, "\\chapter{\\daftitle{This is a title}}");
    }
}
