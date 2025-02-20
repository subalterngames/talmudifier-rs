use crate::{error::Error, font::cosmic_font::CosmicFont, page::Page, word::Word};

use cosmic_text::{Buffer, FontSystem, Shaping};
use table::get_table;
use tex_column::TexColumn;
use width::Width;

pub mod position;
mod table;
mod tex_column;
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
}

impl Column {
    pub fn new(
        words: Vec<Word>,
        cosmic_font: CosmicFont,
        tex_font: &str,
        font_system: FontSystem,
    ) -> Self {
        Self {
            words,
            start: 0,
            cosmic_font,
            tex_font: tex_font.to_string(),
            font_system,
        }
    }

    pub fn get_tex_column<'t>(
        &'t mut self,
        num_lines: usize,
        width: Width,
        page: &Page,
    ) -> Result<Option<&'t [Word]>, Error> {
        // Guess the end index with cosmic.
        match self.get_cosmic_index(num_lines, width, page)? {
            Some(cosmic_index) => self.get_tex_words(width, cosmic_index, num_lines, page),
            None => Err(Error::NoMoreWords),
        }
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

    fn get_num_lines_cosmic<'t>(&'t mut self, end: usize, width: Width, page: &Page) -> usize {
        // Get the width of the column in pts.
        let column_width = page.table_width * width.column_ratio();
        // Prepare the Cosmic buffer.
        let mut buffer = Buffer::new(&mut self.font_system, self.cosmic_font.metrics);
        // Set the width.
        buffer.set_size(&mut self.font_system, Some(column_width), None);
        // Get the Cosmic spans.
        let spans = Word::to_cosmic(&self.words[self.start..end], &self.cosmic_font);
        // Set the text.
        buffer.set_rich_text(
            &mut self.font_system,
            spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
            self.cosmic_font.regular.as_attrs(),
            Shaping::Advanced,
        );
        // Create lines.
        buffer.shape_until_scroll(&mut self.font_system, true);
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
        let (column, title) = Word::to_tex(&self.words[self.start..end], &self.tex_font);

        if title || column.is_empty() {
            Ok(0)
        } else {
            // Get the preamble.
            let mut tex = page.preamble.clone();
            tex.push_str(&get_table(&[TexColumn {
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

    fn get_tex_words<'t>(
        &'t mut self,
        width: Width,
        cosmic_index: usize,
        num_lines: usize,
        page: &Page,
    ) -> Result<Option<&'t [Word]>, Error> {
        if self.start == self.words.len() {
            Ok(None)
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
                None
            } else {
                let start = self.start;
                self.start = if end > self.words.len() {
                    self.words.len()
                } else if end == self.words.len() {
                    end
                } else {
                    end - 1
                };
                Some(&self.words[start..self.start])
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;

    use crate::{
        column::width::Width,
        font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
        page::Page,
        word::Word,
    };

    use super::Column;

    #[test]
    fn test_cosmic_index() {
        let lorem = include_str!("../lorem.txt");
        let words = Word::from_md(lorem).unwrap();
        assert_eq!(words.len(), 402);
        let mut font_system = FontSystem::new();
        let cosmic_font = CosmicFont::default_left(&mut font_system);
        let tex_fonts = TexFonts::default().unwrap();
        let mut column = Column::new(words, cosmic_font, &tex_fonts.left.command, font_system);
        let page = Page::default();
        let cosmic_index = column
            .get_cosmic_index(4, Width::Half, &page)
            .unwrap()
            .unwrap();
        assert_eq!(cosmic_index, 52);
    }
}
