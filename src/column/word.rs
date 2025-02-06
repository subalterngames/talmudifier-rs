use cosmic_text::{Attrs, Weight};
use markdown::{mdast::Node, message::Message, to_mdast, Constructs, ParseOptions};

use crate::tex;

use super::{style::Style, word_position::WordPosition};

pub struct Word {
    pub word: String,
    pub style: Style,
    pub position: WordPosition,
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
        Self::add_node(&node, &mut words, Style::default(), WordPosition::default());
        Ok(words)
    }

    /// Convert a slice of words into Cosmic text spans.
    pub fn to_cosmic<'t>(words: &[Self], mut attrs: Attrs<'t>) -> Vec<(String, Attrs<'t>)> {
        let mut cosmic_spans = vec![];
        let mut span = vec![];
        let mut style = Style::default();
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
                attrs = match word.style {
                    Style::Bold => attrs.weight(Weight::BOLD).style(cosmic_text::Style::Normal),
                    Style::Italic => attrs
                        .weight(Weight::NORMAL)
                        .style(cosmic_text::Style::Italic),
                    Style::BoldItalic => {
                        attrs.weight(Weight::BOLD).style(cosmic_text::Style::Italic)
                    }
                    _ => attrs
                        .weight(Weight::NORMAL)
                        .style(cosmic_text::Style::Normal),
                };
                // Remember the style.
                style = word.style;
            }
        }
        cosmic_spans
    }

    pub fn to_tex(words: &[Self], font_command: &str, citation_command: &str) -> (String, bool) {
        // Try to build a title.
        let title = words
            .iter()
            .filter(|w| w.position == WordPosition::Title)
            .collect::<Vec<&Word>>();
        // Build a column.
        if title.is_empty() {
            let mut text = font_command.to_string();
            let mut style = Style::default();
            let mut position = WordPosition::default();
            for word in words.iter().filter(|w| w.position != WordPosition::Title) {
                // We changed the style.
                if style != word.style {
                    match (style, word.style) {
                        (Style::Regular, Style::Italic) => {
                            text.push_str(" \\textit{");
                        }
                        (Style::Regular, Style::Bold) => {
                            text.push_str(" \\textit{");
                        }
                        (Style::Regular, Style::BoldItalic) => {
                            text.push_str(" \\textit{\\textbf{");
                        }
                        (Style::Italic | Style::Bold, Style::Regular) => {
                            text.push_str("}");
                        }
                        (Style::Italic, Style::Bold) => {
                            text.push_str("}\\textbf{");
                        }
                        (Style::Italic, Style::BoldItalic) => {
                            text.push_str("\\textbf{");
                        }
                        (Style::Bold, Style::Italic) => {
                            text.push_str("}\\textit{");
                        }
                        (Style::Bold, Style::BoldItalic) => {
                            text.push_str("\\textit{");
                        }
                        (Style::BoldItalic, Style::Regular) => {
                            text.push_str("}}");
                        }
                        (Style::BoldItalic, Style::Italic) => {
                            text.push_str("}}\\textit{");
                        }
                        (Style::BoldItalic, Style::Bold) => {
                            text.push_str("}}\\textbf{");
                        }
                        (Style::Regular, Style::Regular)
                        | (Style::Italic, Style::Italic)
                        | (Style::Bold, Style::Bold)
                        | (Style::BoldItalic, Style::BoldItalic) => unreachable!(),
                    }
                    style = word.style;
                }
                // Change the position.
                if position != word.position {
                    match (position, word.position) {
                        (WordPosition::Body, WordPosition::Footnote) => {
                            text.push_str(citation_command);
                            text.push('{');
                        }
                        (WordPosition::Footnote, WordPosition::Body) => {
                            text.push('}');
                        }
                        (_, _) => unreachable!(),
                    }
                    position = word.position;
                }
                text.push_str(&word.word);
                text.push(' ');
            }
            // Pop the dangling space.
            if let Some(' ') = text.chars().last() {
                text.pop().unwrap();
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
    fn add_node(node: &Node, words: &mut Vec<Self>, style: Style, position: WordPosition) {
        match node {
            Node::Root(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, position)),
            Node::FootnoteDefinition(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, WordPosition::Footnote)),
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
                .for_each(|child| Self::add_node(child, words, style, WordPosition::Title)),
            Node::Paragraph(node) => node
                .children
                .iter()
                .for_each(|child| Self::add_node(child, words, style, position)),
            _ => (),
        }
    }

    /// Split a string into words and add them to `words`.
    fn add_words(value: &str, words: &mut Vec<Self>, style: Style, position: WordPosition) {
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
    use crate::column::{style::Style, word_position::WordPosition};

    use super::Word;

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
            assert_eq!(word.position, WordPosition::Body);
        }
    }

    #[test]
    fn test_footnote() {
        let md = "A[^1]\n[^1]: *footnote*";
        let words = Word::from_md(md).unwrap();
        assert_eq!(&words[0].word, "A");
        assert_eq!(words[0].style, Style::Regular);
        assert_eq!(words[0].position, WordPosition::Body);
        assert_eq!(&words[1].word, "footnote");
        assert_eq!(words[1].position, WordPosition::Footnote);
        assert_eq!(words[1].style, Style::Italic);
    }

    #[test]
    fn test_textit() {
        let md = "*This is italic* and this is regular.";
        let words = Word::from_md(md).unwrap();
        let (tex, title) = Word::to_tex(&words, "\\font", "\\citation");
        assert!(!title);
        assert_eq!(tex, "\\font\\textit{This is italic} and this is regular.")
    }
}
