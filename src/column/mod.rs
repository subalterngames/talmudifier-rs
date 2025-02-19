use crate::{
    error::Error,
    font::cosmic_font::CosmicFont,
    page::{Page, WIDTH_PTS},
    tex::{column_type::ColumnType, table::Table},
    word::Word,
};

use cosmic_text::{Buffer, FontSystem, Shaping};
use position::Position;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use width::Width;

mod position;
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

    pub fn get_words<'t>(
        &'t mut self,
        num_lines: usize,
        width: Width,
    ) -> Result<Option<&'t [Word]>, Error> {
        // Guess the end index with cosmic.
        match self.get_cosmic_index(num_lines, width)? {
            Some(cosmic_index) => self.get_tex_words(width, cosmic_index, num_lines),
            None => Err(Error::NoMoreWords),
        }
    }

    pub fn get_min_num_lines(
        left: &mut Self,
        center: &mut Self,
        right: &mut Self,
    ) -> Result<usize, Error> {
        let has_words = [left, center, right]
            .iter()
            .map(|c| !c.words[c.start..].is_empty())
            .collect::<Vec<bool>>();
        // Derive the table from which columns still have words.
        let table = match (has_words[0], has_words[1], has_words[2]) {
            (true, true, true) => Table::Three,
            (true, false, false) | (false, true, false) | (false, false, true) => Table::One,
            (true, true, false) => Table::LeftCenter,
            (true, false, true) => Table::LeftRight,
            (false, true, true) => Table::CenterRight,
            (false, false, false) => {
                return Err(Error::NoMoreWords);
            }
        };
        // Get the column with the least words.
        let num_lines = [left, center, right]
            .into_par_iter()
            .zip(
                [Position::Left, Position::Center, Position::Right]
                    .into_par_iter()
                    .zip(has_words.into_par_iter()),
            )
            .filter_map(|(w, (p, h))| {
                if !h {
                    None
                } else {
                    let width = table.get_width(&p);
                    Some(w.get_num_lines_tex(None, width))
                }
            })
            .collect::<Vec<Result<usize, Error>>>();
        if let Some(err) = num_lines.iter().find_map(|n| match n {
            Ok(n) => None,
            Err(error) => Some(error),
        }) {
            Err(err.clone())
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
                None => Err(Error::MinNumLines),
            }
        }
    }

    fn get_cosmic_index(&mut self, num_lines: usize, width: Width) -> Result<Option<usize>, Error> {
        if num_lines > 0 {
            let num_words = self.words[self.start..].len();
            for i in 0..num_words {
                let num = self.get_num_lines_cosmic(i, width);
                if num > num_lines {
                    return Ok(if i == 0 { None } else { Some(i - 1) });
                }
            }
        }
        Ok(None)
    }

    fn get_num_lines_cosmic<'t>(&'t mut self, end: usize, width: Width) -> usize {
        // Get the width of the column in pts.
        let column_width = self.table_width * width.column_ratio();
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

    fn get_num_lines_tex(&self, end: Option<usize>, width: Width) -> Result<usize, Error> {
        let end = match end {
            Some(end) => end,
            None => self.words.len(),
        };
        let (column, title) = Word::to_tex(&self.words[self.start..end], &self.tex_font);

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
    ) -> Result<Option<&'t [Word]>, Error> {
        if self.start == self.words.len() {
            Ok(None)
        } else {
            let mut end = cosmic_index;

            // Decrement until we have enough lines.
            while end > 0 && self.get_num_lines_tex(Some(end), width)? > num_lines {
                end -= 1;
            }

            // Increment until we go over.
            while end < self.words.len() && self.get_num_lines_tex(Some(end), width)? <= num_lines {
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
    fn test_cosmic() {
        let lorem = include_str!("../lorem.txt");
        let words = Word::from_md(lorem).unwrap();
        assert_eq!(words.len(), 402);
        let mut font_system = FontSystem::new();
        let cosmic_font = CosmicFont::default_left(&mut font_system);
        let tex_fonts = TexFonts::default().unwrap();
        let page = Page::default();
        let preamble = page.get_preamble(&tex_fonts);
        let mut column = Column::new(
            words,
            cosmic_font,
            &tex_fonts.left.command,
            font_system,
            &page,
            &preamble,
        );
        let cosmic_index = column.get_cosmic_index(4, Width::Half).unwrap().unwrap();
        assert_eq!(cosmic_index, 52);
    }
}
