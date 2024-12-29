use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Style, Weight};
use markdown::{tokenize, Block, Span};

mod columns;
mod page;
pub mod prelude;

fn main() {
    let mut font_system = FontSystem::new();
    let metrics = Metrics::new(14.0, 20.0);

    let rtf = to_rtf(include_str!("test.md"));

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.), None);
    // Attributes indicate what font to choose
    let attrs = Attrs::new();
    buffer.set_rich_text(
        &mut font_system,
        rtf.iter().map(|(s, a)| (s.as_str(), *a)),
        attrs,
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut font_system, true);
    println!("{:?}", buffer.layout_runs().count());
}

fn to_rtf(text: &str) -> Vec<(String, Attrs)> {
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
