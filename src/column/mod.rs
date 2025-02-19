use crate::{
    font::cosmic_font::CosmicFont,
    page::{Page, WIDTH_PTS},
    word::Word,
};

use cosmic_text::FontSystem;
use error::Error;
use width::Width;

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
    /// The Cosmic font system.
    font_system: FontSystem,
    /// The width of the table on the page in pts.
    table_width: f32,
    /// The TeX preamble.
    preamble: String,
}

impl Column {
    pub fn new(
        words: Vec<Word>,
        cosmic_font: CosmicFont,
        tex_font: &str,
        font_system: FontSystem,
        page: &Page,
        preamble: &str,
    ) -> Self {
        let table_width = WIDTH_PTS - (page.left_margin.get_pts() + page.right_margin.get_pts());
        Self {
            words,
            start: 0,
            cosmic_font,
            tex_font: tex_font.to_string(),
            font_system,
            table_width,
            preamble: preamble.to_string(),
        }
    }
    
    pub fn get_words<'t>(&'t mut self, num_lines: usize, width: Width) -> Result<Option<&'t [Word]>, Error> {
        let cosmic_index = self.get_cosmic_index(num_lines, width)?;
    }

    fn get_cosmic_index(&mut self, num_lines: usize, width: Width) -> Result<Option<usize>, Error> {
        if num_lines > 0 {
            for i in 0..words.len() {
                // We exceeded the number of lines. Break!
                match self.get_num_lines_cosmic(&self.words[self.start..i], width) {
                    Ok(num) => {
                        if num > num_lines {
                            return Ok(if i == 0 { None } else { Some(i - 1) });
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(None)
    }

    fn get_num_lines_cosmic(&mut self, words: &[Word], width: Width) -> usize {
        // Get the width of the column in pts.
        let column_width = self.table_width * width.column_ratio();
        // Prepare the Cosmic buffer.
        let mut buffer = Buffer::new(&mut self.font_system, self.font.metrics);
        // Set the width.
        buffer.set_size(&mut self.font_system, Some(column_width), None);
        // Get the Cosmic spans.
        let spans = Word::to_cosmic(words, self.font);
        // Set the text.
        buffer.set_rich_text(
            &mut self.font_system,
            spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
            self.font.regular.as_attrs(),
            Shaping::Advanced,
        );
        // Create lines.
        buffer.shape_until_scroll(&mut self.font_system, true);
        // Return the number of lines.
        buffer.layout_runs().count()
    }

    fn get_tex_words<'t>(
        &'t mut self,
        width: Width,
        cosmic_index: usize,
        num_lines: usize,
    ) -> Result<Option<&'t [Word]>, Error> {
        if words.is_empty() {
            Ok(None)
        } else {
            let mut end = cosmic_index;

            // Decrement until we have enough lines.
            while end > 0
                && Self::get_num_lines(&self.preamble, &self.words[self.start..end], &self.tex_font, width)? > num_lines
            {
                end -= 1;
            }

            // Increment until we go over.
            while end < words.len()
                && Self::get_num_lines(&self.preamble, &self.words[self.start..end], &self.tex_font, width)? <= num_lines
            {
                end += 1;
            }

            Ok(if end == 0 {
                None
            } else if end == words.len() {
                self.start = end;
                Some(&self.words[self.start..])
            } else {
                self.start = end - 1;
                Some(&self.words[self.start..])
            })
        }
    }

    fn get_num_lines_tex(&self, width: Width) -> Result<usize, Error> {
        let (column, title) = Word::to_tex(&self.words[self.start..], &self.tex_font);

        if title || column.is_empty() {
            Ok(0)
        } else {
            // Get the preamble.
            let mut tex = self.preamble.to_string();

            // Get a table.
            tex.push_str(&match width {
                Width::Half => Table::get_columns(
                    ColumnType::Text(column),
                    ColumnType::None,
                    ColumnType::Empty,
                ),
                Width::One => {
                    Table::get_columns(ColumnType::Text(column), ColumnType::None, ColumnType::None)
                }
                Width::Third => Table::get_columns(
                    ColumnType::Text(column),
                    ColumnType::Empty,
                    ColumnType::Empty,
                ),
                Width::TwoThirds => Table::get_columns(
                    ColumnType::None,
                    ColumnType::Text(column),
                    ColumnType::Empty,
                ),
            });

            // End the document.
            tex.push_str(Page::END_DOCUMENT);

            // Create a PDF.
            match latex_to_pdf(&tex) {
                // Extract the text of the PDF.
                Ok(pdf) => match extract_text_from_mem(&pdf) {
                    Ok(text) => Ok(text.split('\n').count()),
                    Err(error) => Err(Error::Extract(error)),
                },
                Err(error) => Err(Error::Pdf(error)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;

    use crate::{
        column::{width::Width, ColumnMaker},
        font::cosmic_font::CosmicFont,
        word::Word,
    };

    use super::Cosmic;

    #[test]
    fn test_cosmic() {
        let lorem = include_str!("../lorem.txt");
        let words = Word::from_md(lorem).unwrap();
        assert_eq!(words.len(), 402);
        let mut font_system = FontSystem::new();
        let font = CosmicFont::default_left(&mut font_system);
        let mut cosmic_colunmn = Cosmic::new(&font, Width::Half, 614., &mut font_system);
        let num_lines = cosmic_colunmn.get_num_lines(&words).unwrap();
        assert_eq!(num_lines, 52);
    }
}
