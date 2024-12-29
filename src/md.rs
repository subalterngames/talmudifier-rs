use cosmic_text::{Attrs, Style, Weight};
use markdown::{tokenize, Block, Span};

pub fn to_rtf(text: &str) -> Vec<(String, Attrs)> {
    let attrs = Attrs::new();
    tokenize(text)
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(spans) => Some(get_spans(spans, attrs)),
            _ => None,
        })
        .flatten()
        .collect()
}

fn get_spans<'a>(spans: &[Span], attrs: Attrs<'a>) -> Vec<(String, Attrs<'a>)> {
    spans
        .iter()
        .filter_map(|span| match span {
            Span::Text(text) => Some(vec![(text.clone(), attrs.style(Style::Normal))]),
            Span::Emphasis(spans) => Some(get_spans(spans, attrs.style(Style::Italic))),
            Span::Strong(spans) => Some(get_spans(spans, attrs.weight(Weight::BOLD))),
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {}
