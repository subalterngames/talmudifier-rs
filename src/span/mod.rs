use cosmic_text::AttrsOwned;
use markdown::{mdast::Node, to_mdast, Constructs, ParseOptions};
use position::Position;
use style::Style;
use word::Word;

use crate::{error::Error, font::cosmic_font::CosmicFont};

pub mod position;
pub mod style;
mod word;

type LatexCommand = (Option<&'static str>, Option<&'static str>);

pub struct Span(pub Vec<Word>);

impl Span {
    /// Parse raw markdown text and get a vec of words.
    pub fn from_md(md: &str) -> Result<Self, Error> {
        let parse_options = ParseOptions {
            constructs: Constructs::gfm(),
            ..Default::default()
        };
        match to_mdast(md, &parse_options) {
            Ok(node) => {
                let mut words = vec![];
                // Add the words as nodes.
                Self::add_node(&node, &mut words, Style::default(), Position::default())?;
                Ok(Self(words))
            }
            Err(error) => Err(Error::Md(error)),
        }
    }

    /// A words from a markdown node.
    fn add_node(
        node: &Node,
        words: &mut Vec<Word>,
        style: Style,
        position: Position,
    ) -> Result<(), Error> {
        match node {
            // Add from the root node.
            Node::Root(node) => node
                .children
                .iter()
                .try_for_each(|child| Self::add_node(child, words, style, position)),
            Node::InlineCode(node) => {
                // Treat the inline code as a citation. Create a new node and start to apply TeX commands.
                let parse_options = ParseOptions {
                    constructs: Constructs::gfm(),
                    ..Default::default()
                };
                match to_mdast(&node.value, &parse_options) {
                    Ok(node) => Self::add_node(&node, words, Style::Regular, Position::Margin),
                    Err(error) => Err(Error::Md(error)),
                }
            }
            Node::Emphasis(node) => {
                // Add an italic style.
                let style = match style {
                    Style::Bold => Style::BoldItalic,
                    _ => Style::Italic,
                };
                node.children
                    .iter()
                    .try_for_each(|child| Self::add_node(child, words, style, position))
            }
            Node::Strong(node) => {
                // Add a bold style.
                let style = match style {
                    Style::Italic => Style::BoldItalic,
                    _ => Style::Bold,
                };
                node.children
                    .iter()
                    .try_for_each(|child| Self::add_node(child, words, style, position))
            }
            Node::Text(text) => {
                Self::add_words(&text.value, words, style, position);
                Ok(())
            }
            Node::Paragraph(node) => node
                .children
                .iter()
                .try_for_each(|child| Self::add_node(child, words, style, position)),
            _ => Ok(()),
        }
    }

    /// Split a string into words and add them to `words`.
    fn add_words(value: &str, words: &mut Vec<Word>, style: Style, position: Position) {
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
    use super::Span;

    use super::{Position, Style};

    #[test]
    fn test_words() {
        let md = "Regular *italic* **bold and *italic*** and **this**";
        let span = Span::from_md(md).unwrap();
        assert_eq!(&span.0[0].word, "Regular");
        assert_eq!(span.0[0].style, Style::Regular);

        assert_eq!(&span.0[1].word, "italic");
        assert_eq!(span.0[1].style, Style::Italic);

        assert_eq!(&span.0[2].word, "bold");
        assert_eq!(span.0[2].style, Style::Bold);

        assert_eq!(&span.0[3].word, "and");
        assert_eq!(span.0[3].style, Style::Bold);

        assert_eq!(&span.0[4].word, "italic");
        assert_eq!(span.0[4].style, Style::BoldItalic);

        assert_eq!(&span.0[5].word, "and");
        assert_eq!(span.0[5].style, Style::Regular);

        assert_eq!(&span.0[6].word, "this");
        assert_eq!(span.0[6].style, Style::Bold);

        for word in span.0.iter() {
            assert_eq!(word.position, Position::Body);
        }
    }

    #[test]
    fn test_footnote() {
        let md = "A `footnote` *here*";
        let span = Span::from_md(md).unwrap();
        assert_eq!(&span.0[0].word, "A");
        assert_eq!(span.0[0].style, Style::Regular);
        assert_eq!(span.0[0].position, Position::Body);
        assert_eq!(&span.0[1].word, "footnote");
        assert_eq!(span.0[1].position, Position::Margin);
        assert_eq!(span.0[1].style, Style::Regular);
        assert_eq!(span.0[2].position, Position::Body);
        assert_eq!(span.0[2].style, Style::Italic);
    }

    #[test]
    fn test_textit() {
        let md = "*This is italic* and this is regular.";
        let span = Span::from_md(md).unwrap();
        let tex = span.to_tex("\\font");
        assert_eq!(tex, "\\font \\textit{This is italic} and this is regular.")
    }

    #[test]
    fn test_bold_italic() {
        let md = "**bold** *italic* ***bold and italic*** **bold**";
        let span = Span::from_md(md).unwrap();
        let tex = span.to_tex("\\font");
        assert_eq!(
            tex,
            "\\font \\textbf{bold} \\textit{italic \\textbf{bold and italic}} \\textbf{bold}"
        )
    }

    #[test]
    fn test_marginnote() {
        let md = "A `footnote *here* and` *there*";
        let span = Span::from_md(md).unwrap();
        let tex = span.to_tex("\\font");
        assert_eq!(
            tex,
            "\\font A \\\\marginnote{\\\\noindent\\\\justifying\\\\tiny footnote \\textit{here} and} \\textit{there}"
        );
    }
}
