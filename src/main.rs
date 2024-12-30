use std::{path::PathBuf, str::FromStr};

use cosmic_text::{
    fontdb::Source, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, Weight,
};
use markdown::{tokenize, Block, Span};
use tex_span::TexSpan;

mod columns;
mod page;
pub mod prelude;
mod tex_span;

fn main() {
    let mut font_system = FontSystem::new();
    let metrics = Metrics::new(14.0, 20.0);

    let rtf = to_rtf(include_str!("test.md"));

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.), None);

    let path = PathBuf::from_str("src/fonts/IM_Fell_French_Canon/FeFCit2.ttf").unwrap();
    assert!(path.exists(), "{:?}", path);
    let font_id = font_system.db_mut().load_font_source(Source::File(path))[0];

    let path = PathBuf::from_str("src/fonts/IM_Fell_French_Canon/FeFCrm2.ttf").unwrap();
    println!("{:?}", &font_system.db().face(font_id).unwrap());
    let font_id = font_system.db_mut().load_font_source(Source::File(path))[0];

    let family_name = font_system.db().face(font_id).unwrap().families[0]
        .0
        .clone();
    // Attributes indicate what font to choose
    let attrs = Attrs::new().family(Family::Name(&family_name));
    buffer.set_rich_text(
        &mut font_system,
        rtf.iter().map(|(s, a)| (s.as_str(), *a)),
        attrs,
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut font_system, true);
    println!("{:?}", buffer.layout_runs().count());
}

fn to_tex(text: &str) -> Vec<TexSpan> {
    to_rtf(text).iter().map(|value| value.into()).collect()
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
