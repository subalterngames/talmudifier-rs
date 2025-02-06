use markdown::{mdast::Node, message::Message, to_mdast};

use super::{style::Style, word_position::WordPosition};

pub struct Word {
    pub word: String,
    pub style: Style,
    pub position: WordPosition,
}

impl Word {
    pub fn from_md(md: &str) -> Result<Vec<Self>, Message> {
        let node = to_mdast(md, &Default::default())?;
        let mut words = vec![];
        // Add the words as nodes.
        Self::add_node(&node, &mut words, Style::default(), WordPosition::default());
        Ok(words)
    }

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
            Node::InlineCode(node) => Self::add_words(&node.value, words, Style::Code, position),
            Node::InlineMath(node) => Self::add_words(&node.value, words, Style::Code, position),
            Node::Emphasis(node) => {
                // Add an italic style.
                let style = match style {
                    Style::Bold => Style::BoldItalic,
                    Style::Code => Style::Code,
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
                    Style::Code => Style::Code,
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
        let md = "Regular *italic* **bold and *italic*** and ***bold and*** *italic*";
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
        for word in words.iter() {
            assert_eq!(word.position, WordPosition::Body);
        }
    }
}
