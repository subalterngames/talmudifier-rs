use std::{path::PathBuf, str::FromStr};

use column_position::ColumnPosition;
use columns::Columns;
use cosmic_text::{
    fontdb::Source, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, Weight,
};
use index::Index;
use markdown::{tokenize, Block, Span};
use page::{paper_size::WIDTH_PTS, Page};
use tex_span::TexSpan;

mod column_position;
mod columns;
mod index;
mod page;
pub mod prelude;
mod tex_span;

fn main() {
    let mut font_system = FontSystem::new();
    let metrics = Metrics::new(14.0, 20.0);

    let rtf = to_cosmic(include_str!("test.md"));

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
}

fn to_tex(text: &str) -> Vec<TexSpan> {
    to_cosmic(text).iter().map(|value| value.into()).collect()
}

fn to_cosmic(text: &str) -> Vec<(Vec<String>, Attrs)> {
    let attrs = Attrs::new();
    tokenize(text)
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(spans) => Some(get_spans(spans, attrs)),
            _ => None,
        })
        .flatten()
        .map(|(s, a)| (s.split(' ').map(|s| s.to_string()).collect(), a))
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

fn get_cosmic_index(
    spans: &[(Vec<String>, Attrs)],
    num_rows: usize,
    position: &ColumnPosition,
    columns: &Columns,
    page: &Page,
    font_system: &mut FontSystem,
    default_attrs: Attrs,
    metrics: Metrics,
) -> Option<Index> {
    // Get the total width of the columns.
    let total_width = WIDTH_PTS - (page.left_margin.get_pts() + page.right_margin.get_pts());
    // Get the column width.
    let column_width = total_width * columns.get_width(position);

    // Get the buffer.
    let mut buffer = Buffer::new(font_system, metrics);
    // Set the width.
    buffer.set_size(font_system, Some(column_width), None);

    // Get the approximate width of the cell.
    let mut index = Index::default();

    // Add words to this.
    let mut cosmic_spans: Vec<(String, Attrs)> = vec![];

    for (i, (span, attrs)) in spans.iter().enumerate() {
        // Set the span index.
        index.span = i;
        let mut words = vec![];
        for (j, word) in span.iter().enumerate() {
            // Add the word.
            words.push(word.clone());
            // Add the span.
            cosmic_spans.push((words.join(" "), *attrs));

            // Set the text.
            buffer.set_rich_text(
                font_system,
                cosmic_spans.iter().map(|(s, a)| (s.as_str(), *a)),
                default_attrs,
                Shaping::Advanced,
            );
            // Create lines.
            buffer.shape_until_scroll(font_system, true);
            let num_lines = buffer.layout_runs().count();

            // Not enough lines. Continue.
            if num_rows <= num_lines {
                index.word = j;
            }
            // We exceedded the number of lines. Remove the last word and return.
            else {
                // Remove the word.
                let _ = words.pop();
                // Remove the span.
                let _ = cosmic_spans.pop();
                // Add a span.
                cosmic_spans.push((words.join(" "), *attrs));
                return Some(index);
            }
        }
    }

    if cosmic_spans.is_empty() {
        None
    } else {
        Some(index)
    }
}
