use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Style, Weight};
use markdown::{tokenize, Block, Span};

use crate::{
    column::width::Width,
    index::Index,
    page::{paper_size::WIDTH_PTS, Page},
};

pub struct Typesetter<'t> {
    md: String,
    num_rows: usize,
    column_ratio: f32,
    column_width: f32,
    attrs: Attrs<'t>,
}

impl<'t> Typesetter<'t> {
    pub fn new(md: String, num_rows: usize, attrs: Attrs<'t>, page: &Page, width: &Width) -> Self {
        let column_ratio = width.column_ratio();
        // Get the total width of the columns.
        let total_width = WIDTH_PTS - (page.left_margin.get_pts() + page.right_margin.get_pts());
        // Get the column width.
        let column_width = total_width * column_ratio;

        Self {
            md,
            num_rows,
            column_ratio,
            column_width,
            attrs,
        }
    }

    /// Iterate through spans of text to get a block of text that runs for a given number of lines.
    /// Cosmic won't format text the same way as TeX will.
    /// The result of this function is used as a baseline for typesetting with TeX, which is much slower.
    ///
    /// - `num_rows`: The target number of rows.
    /// - `column`: The column of text.
    /// - `columns`: All columns on the page. This is used to determine the column width.
    /// - `page`: Page values. This is used to determine the column width.
    /// - `font_system`: Cosmic text font system.
    /// - `metrics`: Font metrics, namely the font size.
    fn guess_index(&self, font_system: &mut FontSystem, metrics: Metrics) -> Option<Index> {
        // Get the buffer.
        let mut buffer = Buffer::new(font_system, metrics);
        // Set the width.
        buffer.set_size(font_system, Some(self.column_width), None);

        // Get the cosmic spans.
        let spans = self.markdown_to_cosmic();

        // The spans that we've included so far.
        let mut finished_spans = vec![];

        let mut index = Index::default();

        for (i, (span, attrs)) in spans.iter().enumerate() {
            // Set the span index.
            index.span = i;
            let mut words = vec![];
            for (j, word) in span.iter().enumerate() {
                // Add the word.
                words.push(word.clone());
                // Add the span.
                finished_spans.push((words.join(" "), *attrs));

                // Set the text.
                buffer.set_rich_text(
                    font_system,
                    finished_spans.iter().map(|(s, a)| (s.as_str(), *a)),
                    self.attrs,
                    Shaping::Advanced,
                );
                // Create lines.
                buffer.shape_until_scroll(font_system, true);
                let num_lines = buffer.layout_runs().count();

                // Not enough lines. Continue.
                if self.num_rows <= num_lines {
                    index.word = j;
                }
                // We exceedded the number of lines. Remove the last word and return.
                else {
                    // Remove the word.
                    let _ = words.pop();
                    // Remove the span.
                    let _ = finished_spans.pop();
                    // Add the revised span.
                    finished_spans.push((words.join(" "), *attrs));
                    return Some(index);
                }
            }
        }

        if finished_spans.is_empty() {
            None
        } else {
            Some(index)
        }
    }

    /// Convert a raw markdown string to Cosmic text spans.
    fn markdown_to_cosmic(&self) -> Vec<(Vec<String>, Attrs)> {
        tokenize(&self.md)
            .iter()
            .filter_map(|block| match block {
                Block::Paragraph(spans) => {
                    Some(self.markdown_paragraph_to_cosmic(spans, self.attrs))
                }
                _ => None,
            })
            .flatten()
            .map(|(s, a)| (s.split(' ').map(|s| s.to_string()).collect(), a))
            .collect()
    }

    /// Convert multiple spans in a markdown paragraph into Cosmic text spans.
    fn markdown_paragraph_to_cosmic(
        &self,
        spans: &[Span],
        attrs: Attrs<'t>,
    ) -> Vec<(String, Attrs<'t>)> {
        spans
            .iter()
            .filter_map(|span| match span {
                Span::Text(text) => Some(vec![(text.clone(), attrs.style(Style::Normal))]),
                Span::Emphasis(spans) => {
                    Some(self.markdown_paragraph_to_cosmic(spans, attrs.style(Style::Italic)))
                }
                Span::Strong(spans) => {
                    Some(self.markdown_paragraph_to_cosmic(spans, attrs.weight(Weight::BOLD)))
                }
                _ => None,
            })
            .flatten()
            .collect()
    }
}
