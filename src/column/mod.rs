use std::cmp::Ordering;

use crate::{error::Error, font::cosmic_font::CosmicFont, page::Page, word::Word};

use cosmic_text::{Buffer, Shaping};
use input_column::InputColumn;
#[cfg(not(target_os = "windows"))]
use pdf_extract::extract_text_from_mem;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};
#[cfg(not(target_os = "windows"))]
use tectonic::latex_to_pdf;
use tex_column::TexColumn;
use width::Width;

pub mod input_column;
pub mod tex_column;
pub mod width;

/// A column of text that can be typeset.
/// Columns try to fill a target number of lines with words.
/// Cosmic is used to get an initial guess at the number of words.
/// Then, Tectonic is used to fill the column.
///
/// `Column` has a `Vec<Word>` and a `start` index that are continuously re-sliced for typesetting.
/// Every time `get_tex_table(left, center, right, num_lines, page)`  is called, `start` is incremented.
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

    /// Given 1-3 columns and a target `num_lines`, create a TeX table.
    ///
    /// `left`, `center`, and `right` are the columns. At least one of them must be `Empty` or `Text`.
    pub fn get_tex_table<'t>(
        left: &mut InputColumn<'t>,
        center: &mut InputColumn<'t>,
        right: &mut InputColumn<'t>,
        num_lines: usize,
        page: &Page,
    ) -> Result<Vec<TexColumn>, Error> {
        if !left.is_column() && !center.is_column() && !right.is_column() {
            return Err(Error::NoColumns);
        }
        // Get the width of each column and convert them into empty TexColumns.
        let mut tex_columns =
            Self::get_widths(left.is_column(), center.is_column(), right.is_column())
                .into_iter()
                .map(|width| TexColumn { width, text: None })
                .collect::<Vec<TexColumn>>();

        // Get a tex string per column.
        [left, center, right]
            .par_iter_mut()
            .zip(tex_columns.par_iter_mut())
            .try_for_each(|(input_column, tex_column)| match input_column {
                // This is not a column.
                InputColumn::None => unreachable!(),
                // This is an empty column.
                InputColumn::Empty => Ok(()),
                // This is a column with text.
                InputColumn::Text(input_column) => {
                    // Fill the column with text.
                    *tex_column = input_column.get_tex_column(num_lines, tex_column.width, page)?;
                    Ok(())
                }
            })?;
        Ok(tex_columns)
    }

    /// Fill a column of `num_lines` with a TeX string.
    /// Estimate an end index for `self.words` with Cosmic Text.
    /// Use that estimate to typeset with Tectonic.
    fn get_tex_column(
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

    /// Given a target number of lines, typeset using Cosmic.
    /// Returns the end index of the words that fit in the column.
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

    /// Get the number of lines in a column with Cosmic Text.
    ///
    /// - `end` is an optional end index for `self.words`. If `None`, the words are from `self.start..self.words.len()`.
    /// - `width` is the width of the column, as calculated elsewhere.
    /// - `page` is used because we need width of the table.
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

    /// Get the number of lines in a column using Tectonic.
    ///
    /// - `end` is an optional end index for `self.words`. If `None`, the words are from `self.start..self.words.len()`.
    /// - `width` is the width of the column, as calculated elsewhere.
    /// - `page` is used because we need the preamble.
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
            Err(Error::Pdf)
        }
    }

    /// Typeset using Tectonic to fill a column with `num_lines` of a TeX string.
    /// `cosmic_index` is the best-first-guess of the end index.
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
                    // If we overshot the number of words, use words.len()
                    Ordering::Greater => self.words.len(),
                    // Use the end index
                    Ordering::Equal => end,
                    // This will happen if we needed to increment.
                    Ordering::Less => end - 1,
                };
                // Convert words to a TeX string.
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

    /// Given 1-3 columns:
    ///
    /// 1. Get the width of each column.
    ///    This is derived from the position of each column e.g. `left` and `center` is always (one third, two thirds).
    /// 2. Get the number of lines per column, using all words that have yet to be typeset.
    /// 3. Return the least number of lines. This will be used as a target for all columns to reach.
    pub fn get_min_num_lines(
        left: Option<&Self>,
        center: Option<&Self>,
        right: Option<&Self>,
        page: &Page,
    ) -> Result<usize, Error> {
        if left.is_none() && center.is_none() && right.is_none() {
            return Err(Error::MinNumLines("All columns are None".to_string()));
        }

        // Get the width of each line.
        let widths = Self::get_widths(left.is_some(), center.is_some(), right.is_some());

        // Get all non-None columns.
        let columns = [left, center, right]
            .into_iter()
            .flatten()
            .collect::<Vec<&Self>>();

        // Get the columns that still have words that need to be typeset.
        let has_words = columns.iter().map(|c| !c.done()).collect::<Vec<bool>>();

        // Get the column with the least words.
        let mut num_lines = vec![0; columns.len()];
        columns
            .into_par_iter()
            .zip(
                has_words
                    .into_par_iter()
                    .zip(widths.into_par_iter().zip(num_lines.par_iter_mut())),
            )
            .try_for_each(|(column, (has_words, (width, num_lines)))| {
                if has_words {
                    match column.get_num_lines_tex(None, width, page) {
                        Ok(n) => {
                            *num_lines = n;
                            Ok(())
                        }
                        Err(error) => Err(error),
                    }
                } else {
                    Ok(())
                }
            })?;
        match num_lines.iter().min() {
            Some(min) => Ok(*min),
            None => Err(Error::MinNumLines("Called min() but got None".to_string())),
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

        let cosmic_index = column
            .get_cosmic_index(4, Width::One, &page)
            .unwrap()
            .unwrap();
        assert_eq!(cosmic_index, 46);
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

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_num_lines() {
        let (left, _, _) = get_test_md();

        let tex_fonts = TexFonts::default().unwrap();
        let left = get_column(&left, &tex_fonts.left.command, CosmicFont::default_left);
        let num_lines = left
            .get_num_lines_tex(None, Width::Half, &Page::default())
            .unwrap();
        assert_eq!(num_lines, 22);
    }

    //#[test]
    #[cfg(not(target_os = "windows"))]
    fn test_min_num_lines() {
        let (left, center, right) = get_columns();
        let min_num_lines =
            Column::get_min_num_lines(Some(&left), Some(&center), Some(&right), &Page::default())
                .unwrap();
        assert_eq!(min_num_lines, 4);
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
