use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Style, Weight};
use markdown::{tokenize, Block, Span};
use pdf_extract::extract_text_from_mem;
use tectonic::latex_to_pdf;

use crate::{
    column::width::Width,
    font_family::FontFamily,
    index::Index,
    page::{paper_size::WIDTH_PTS, Page},
    tex,
    tex_span::TexSpan,
};

use error::PdfError;

mod error;

pub struct Typesetter<'t> {
    num_rows: usize,
    column_ratio: f32,
    column_width: f32,
    attrs: Attrs<'t>,
    start_index: Index,
    metrics: Metrics,
    preamble: String,
    font_command: String,
    cosmic_spans: Vec<(Vec<String>, Attrs<'t>)>,
    tex_spans: Vec<TexSpan>,
}

impl<'t> Typesetter<'t> {
    const END_PARACOL: &'static str = "\\end{paracol}";

    pub fn new(
        md: &str,
        num_rows: usize,
        attrs: Attrs<'t>,
        page: &Page,
        width: &Width,
        font_family: FontFamily,
        start_index: Index,
    ) -> Self {
        let column_ratio = width.column_ratio();
        // Get the total width of the columns.
        let total_width = WIDTH_PTS - (page.left_margin.get_pts() + page.right_margin.get_pts());
        // Get the column width.
        let column_width = total_width * column_ratio;

        let mut preamble = page.get_preamble();
        // Add the font family declaration.
        preamble.push_str(&font_family.font_family);
        preamble.push_str(Page::BEGIN_DOCUMENT);

        // Add the paracol.
        preamble.push_str(&tex!("columnratio", column_ratio));
        preamble.push('\n');
        preamble.push_str(&tex!("begin", "paracol", 1));
        preamble.push_str(&font_family.command);
        preamble.push(' ');

        // Get the spans.
        let cosmic_spans = Self::get_cosmic_spans(md, attrs);
        let tex_spans = cosmic_spans.iter().map(|value| value.into()).collect();

        Self {
            num_rows,
            column_ratio,
            column_width,
            attrs,
            start_index,
            preamble,
            metrics: font_family.metrics,
            font_command: font_family.command,
            cosmic_spans,
            tex_spans,
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
    fn guess_end_index(&self, font_system: &mut FontSystem) -> Option<Index> {
        // Get the buffer.
        let mut buffer = Buffer::new(font_system, self.metrics);
        // Set the width.
        buffer.set_size(font_system, Some(self.column_width), None);

        // The spans that we've included so far.
        let mut finished_spans = vec![];

        for (i, (span, attrs)) in self.cosmic_spans[self.start_index.span..]
            .iter()
            .enumerate()
        {
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

                // We exceedded the number of lines. Remove the last word and return.
                if self.num_rows > num_lines {
                    // Remove the word.
                    let _ = words.pop();
                    // Remove the span.
                    let _ = finished_spans.pop();
                    // Add the revised span.
                    finished_spans.push((words.join(" "), *attrs));
                    return Some(Index {
                        span: i,
                        word: j,
                        hyphen: None,
                    });
                }
            }
        }

        // Return everything or nothing.
        match self.cosmic_spans.last() {
            Some(span) => Some(Index {
                span: self.cosmic_spans.len(),
                word: span.0.len(),
                hyphen: None,
            }),
            None => None,
        }
    }

    pub fn get_tex(&self, end_index: &mut Index) -> Result<String, PdfError> {
        let mut decremented = false;
        let mut result = None;
        let mut decremented = false;
        while let None = result {
            // Try to fill the cell with spans.
            let content = self.tex_spans[self.start_index.span..end_index.span]
                .iter()
                .map(|t| t.get_tex(None))
                .collect::<Vec<String>>()
                .join(" ");
            match self.get_num_lines(end_index) {
                Ok(num_lines) => {
                    // Not enough rows. Increment the end index.
                    if num_lines < self.num_rows {
                        // Stop decrementing.
                        if !decremented {
                            decremented = true;
                        }
                        // Increment the span index.
                        end_index.span += 1;
                        // No more words.
                        if end_index.span == self.tex_spans.len() {
                            result = Some(Ok(self.tex_spans[self.start_index.span..]
                                .iter()
                                .map(|t| t.get_tex(None))
                                .collect::<Vec<String>>()
                                .join(" ")));
                        }
                    }
                    // Try to decrement the number of spans.
                    else if num_lines > self.num_rows && !decremented {
                        end_index.span -= 1;
                    }
                    // We're done incrementing the span. Let's set the word.
                    else {
                        let mut tex_spans = self.tex_spans[self.start_index.span..end_index.span].to_vec();
                        // Remove some words.
                        if let Some(span) = tex_spans.pop() {
                            let content = self.tex_spans[self.start_index.span..end_index.span]
                            .iter()
                            .map(|t| t.get_tex(None))
                            .collect::<Vec<String>>();
                            content.push(span.get_tex(index));
                        }
   
                    }
                }
                Err(error) => {
                    result = Some(Err(error));
                }
            }
        }
        match result {
            Some(result) => result,
            None => unreachable!()
        }
    }

    pub fn guess_and_get_tex(&mut self, font_system: &mut FontSystem) -> Option<String> {
        match self.guess_index(font_system) {
            Some(end_index) => Some(self.get_tex(end_index)),
            None => None,
        }
    }

    /// Convert a raw markdown string to Cosmic text spans.
    fn get_cosmic_spans(md: &str, attrs: Attrs<'t>) -> Vec<(Vec<String>, Attrs<'t>)> {
        tokenize(md)
            .iter()
            .filter_map(|block| match block {
                Block::Paragraph(spans) => {
                    Some(Self::get_cosmic_spans_from_paragraph(spans, attrs))
                }
                _ => None,
            })
            .flatten()
            .map(|(s, a)| (s.split(' ').map(|s| s.to_string()).collect(), a))
            .collect()
    }

    /// Convert multiple spans in a markdown paragraph into Cosmic text spans.
    fn get_cosmic_spans_from_paragraph(
        spans: &[Span],
        attrs: Attrs<'t>,
    ) -> Vec<(String, Attrs<'t>)> {
        spans
            .iter()
            .filter_map(|span| match span {
                Span::Text(text) => Some(vec![(text.clone(), attrs.style(Style::Normal))]),
                Span::Emphasis(spans) => Some(Self::get_cosmic_spans_from_paragraph(
                    spans,
                    attrs.style(Style::Italic),
                )),
                Span::Strong(spans) => Some(Self::get_cosmic_spans_from_paragraph(
                    spans,
                    attrs.weight(Weight::BOLD),
                )),
                _ => None,
            })
            .flatten()
            .collect()
    }

    fn get_num_lines(&self, end_index: &Index) -> Result<usize, Error> {
        // Get the tex doc.
        let doc = format!(
            "{}{}\n\\end{{paracol}}\n{}",
            &self.preamble,
            &content,
            Page::END_DOCUMENT
        );
        match latex_to_pdf(&doc) {
            Ok(pdf) => match extract_text_from_mem(&pdf) {
                Ok(pdf) => Ok(pdf.split("\n").count()),
                Err(error) => PdfError::Extraction(error),
            },
            Err(error) => return Err(error),
        }
    }
}
