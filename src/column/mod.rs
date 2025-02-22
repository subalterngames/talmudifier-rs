use std::cmp::Ordering;

use crate::{error::Error, font::cosmic_font::CosmicFont, page::Page, word::Word};

use cosmic_text::{Buffer, Shaping};
use input_column::InputColumn;
#[cfg(not(target_os = "windows"))]
use pdf_extract::extract_text_from_mem;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
#[cfg(not(target_os = "windows"))]
use tectonic::latex_to_pdf;
use tex_column::TexColumn;
use width::Width;

pub mod input_column;
pub mod tex_column;
pub mod width;

pub struct Column {
    /// All of the words in the column.
    words: Vec<Word>,
    /// The start index of the `words` slice.
    start: usize,
    /// The font used in Cosmic.
    cosmic_font: CosmicFont,
    /// The command to set the TeX font.
    tex_font: String,
}

impl Column {
    pub fn new(words: Vec<Word>, cosmic_font: CosmicFont, tex_font: &str) -> Self {
        Self {
            words,
            start: 0,
            cosmic_font,
            tex_font: tex_font.to_string(),
        }
    }

    pub fn get_tex_table<'t>(
        left: &mut InputColumn<'t>,
        center: &mut InputColumn<'t>,
        right: &mut InputColumn<'t>,
        num_lines: usize,
        page: &Page,
    ) -> Result<Vec<TexColumn>, Error> {
        // Get the width of each column.
        let widths = Self::get_widths(left.is_column(), center.is_column(), right.is_column());

        // Get a tex string per column.
        let results = [left, center, right]
            .iter_mut()
            .zip(widths)
            .filter_map(|(column, width)| match column {
                // This is not a column.
                InputColumn::None => None,
                // This is an empty column.
                InputColumn::Empty => Some(Ok(TexColumn { text: None, width })),
                // This is a column with text.
                InputColumn::Text(column) => Some(column.get_tex_column(num_lines, width, page)),
            })
            .collect::<Vec<Result<TexColumn, Error>>>();

        // Return the first error.
        if results.iter().any(|t| t.is_err()) {
            results
                .into_iter()
                .find_map(|t| match t {
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                })
                .unwrap()
        } else {
            // Return the columns.
            Ok(results.into_iter().flatten().collect())
        }
    }

    pub fn get_tex_column(
        &mut self,
        num_lines: usize,
        width: Width,
        page: &Page,
    ) -> Result<TexColumn, Error> {
        // Guess the end index with cosmic.
        match self.get_cosmic_index(num_lines, width, page)? {
            Some(cosmic_index) => self.get_tex_words(width, cosmic_index, num_lines, page),
            None => Err(Error::NoMoreWords),
        }
    }

    pub fn done(&self) -> bool {
        self.start >= self.words.len()
    }

    fn get_cosmic_index(
        &mut self,
        num_lines: usize,
        width: Width,
        page: &Page,
    ) -> Result<Option<usize>, Error> {
        if num_lines > 0 {
            let num_words = self.words[self.start..].len();
            for i in 0..num_words {
                let num = self.get_num_lines_cosmic(i, width, page);
                if num > num_lines {
                    return Ok(if i == 0 { None } else { Some(i - 1) });
                }
            }
        }
        Ok(None)
    }

    fn get_num_lines_cosmic(&mut self, end: usize, width: Width, page: &Page) -> usize {
        // Get the width of the column in pts.
        let column_width = page.table_width * width.column_ratio();
        // Prepare the Cosmic buffer.
        let mut buffer = Buffer::new(&mut self.cosmic_font.font_system, self.cosmic_font.metrics);
        // Set the width.
        buffer.set_size(&mut self.cosmic_font.font_system, Some(column_width), None);
        // Get the Cosmic spans.
        let spans = Word::to_cosmic(&self.words[self.start..end], &self.cosmic_font);
        // Set the text.
        buffer.set_rich_text(
            &mut self.cosmic_font.font_system,
            spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
            self.cosmic_font.regular.as_attrs(),
            Shaping::Advanced,
        );
        // Create lines.
        buffer.shape_until_scroll(&mut self.cosmic_font.font_system, true);
        // Return the number of lines.
        buffer.layout_runs().count()
    }

    fn get_num_lines_tex(
        &self,
        end: Option<usize>,
        width: Width,
        page: &Page,
    ) -> Result<usize, Error> {
        let end = match end {
            Some(end) => end,
            None => self.words.len(),
        };
        let column = Word::to_tex(&self.words[self.start..end], &self.tex_font);

        if column.is_empty() {
            Ok(0)
        } else {
            // Get the preamble.
            let mut tex = page.preamble.clone();
            tex.push_str(&TexColumn::get_table(&[TexColumn {
                text: Some(column),
                width,
            }]));

            // End the document.
            tex.push_str(Page::END_DOCUMENT);

            // Create a PDF.
            #[cfg(not(target_os = "windows"))]
            match latex_to_pdf(&tex) {
                // Extract the text of the PDF.
                Ok(pdf) => match extract_text_from_mem(&pdf) {
                    Ok(text) => Ok(text.split('\n').count()),
                    Err(error) => Err(Error::Extract(error)),
                },
                Err(error) => Err(Error::Pdf(error)),
            }

            #[cfg(target_os = "windows")]
            panic!("Cannot render PDFs on Windows");
        }
    }

    fn get_tex_words(
        &mut self,
        width: Width,
        cosmic_index: usize,
        num_lines: usize,
        page: &Page,
    ) -> Result<TexColumn, Error> {
        if self.start == self.words.len() {
            Ok(TexColumn { text: None, width })
        } else {
            let mut end = cosmic_index;

            // Decrement until we have enough lines.
            while end > 0 && self.get_num_lines_tex(Some(end), width, page)? > num_lines {
                end -= 1;
            }

            // Increment until we go over.
            while end < self.words.len()
                && self.get_num_lines_tex(Some(end), width, page)? <= num_lines
            {
                end += 1;
            }

            Ok(if end == 0 {
                TexColumn { text: None, width }
            } else {
                let start = self.start;
                self.start = match end.cmp(&self.words.len()) {
                    Ordering::Greater => self.words.len(),
                    Ordering::Equal => end,
                    Ordering::Less => end - 1,
                };
                let text = Word::to_tex(&self.words[start..self.start], &self.tex_font);
                TexColumn {
                    text: Some(text),
                    width,
                }
            })
        }
    }

    /// Returns the width of the left, center, and right columns.
    /// Only returns widths for the columns that aren't `None`.
    fn get_widths(left: bool, center: bool, right: bool) -> Vec<Width> {
        match (left, center, right) {
            (true, true, true) => vec![Width::Third; 3],
            (true, true, false) => vec![Width::Third, Width::TwoThirds],
            (true, false, true) => vec![Width::Half; 2],
            (true, false, false) | (false, true, false) | (false, false, true) => vec![Width::One],
            (false, true, true) => vec![Width::TwoThirds, Width::Third],
            (false, false, false) => vec![],
        }
    }

    pub fn get_min_num_lines(
        left: Option<&Self>,
        center: Option<&Self>,
        right: Option<&Self>,
        page: &Page,
    ) -> Result<usize, Error> {
        let widths = Self::get_widths(left.is_some(), center.is_some(), right.is_some());
        let columns = [left, center, right]
            .into_iter()
            .flatten()
            .collect::<Vec<&Self>>();
        let has_words = columns
            .iter()
            .map(|c| !c.words[c.start..].is_empty())
            .collect::<Vec<bool>>();
        // Get the column with the least words.
        let num_lines = columns
            .into_par_iter()
            .zip(has_words.into_par_iter().zip(widths.into_par_iter()))
            .filter_map(|(column, (has_words, width))| {
                if !has_words {
                    None
                } else {
                    Some(column.get_num_lines_tex(None, width, page))
                }
            })
            .collect::<Vec<Result<usize, Error>>>();
        if let Some(error) = num_lines.iter().find_map(|n| match n {
            Ok(_) => None,
            Err(error) => Some(error),
        }) {
            Err(Error::MinNumLines(error.to_string()))
        } else {
            match num_lines
                .iter()
                .filter_map(|n| match n {
                    Ok(n) => Some(n),
                    Err(_) => None,
                })
                .min()
            {
                Some(min) => Ok(*min),
                None => Err(Error::NoMinNumLines),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        column::width::Width,
        font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
        page::Page,
        tests::get_test_md,
        word::Word,
    };

    use super::Column;

    #[test]
    fn test_cosmic_index() {
        let lorem = include_str!("../../test_text/lorem.txt");
        let words = Word::from_md(lorem).unwrap();
        assert_eq!(words.len(), 402);
        let cosmic_font = CosmicFont::default_left();
        let tex_fonts = TexFonts::default().unwrap();
        let mut column = Column::new(words, cosmic_font, &tex_fonts.left.command);
        let page = Page::default();
        let cosmic_index = column
            .get_cosmic_index(4, Width::Half, &page)
            .unwrap()
            .unwrap();
        assert_eq!(cosmic_index, 22);
    }

    #[test]
    fn test_widths() {
        let widths = Column::get_widths(true, true, true);
        assert_eq!(widths.len(), 3);
        assert!(widths.iter().all(|w| *w == Width::Third));

        let widths = Column::get_widths(true, false, true);
        assert_eq!(widths.len(), 2);
        assert!(widths.iter().all(|w| *w == Width::Half));

        let widths = Column::get_widths(true, false, false);
        assert_eq!(widths.len(), 1);
        assert!(widths.iter().all(|w| *w == Width::One));
    }

    fn get_columns() -> (Column, Column, Column) {
        let (left, center, right) = get_test_md();

        let tex_fonts = TexFonts::default().unwrap();
        let left = get_column(&left, &tex_fonts.left.command, CosmicFont::default_left);
        let center = get_column(&center, &tex_fonts.left.command, CosmicFont::default_left);
        let right = get_column(&right, &tex_fonts.left.command, CosmicFont::default_left);
        (left, center, right)
    }

    fn get_column(md: &str, tex_font: &str, f: impl Fn() -> CosmicFont) -> Column {
        let words = Word::from_md(&md).unwrap();
        let cosmic_font = f();
        Column::new(words, cosmic_font, tex_font)
    }
}
