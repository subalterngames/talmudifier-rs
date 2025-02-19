use std::{
    path::Path,
    thread::{spawn, JoinHandle},
};

use column_type::ColumnType;
use error::Error;
use page::Page;
use pdf_extract::extract_text_from_mem;
use position::Position;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use table::Table;
use tectonic::latex_to_pdf;

use crate::{
    column::width::Width,
    error::Error,
    font::tex_fonts::TexFonts,
    word::Word,
};

pub(crate) mod column_type;
mod length;
pub(crate) mod page;
mod paper_size;
mod position;
pub(crate) mod table;
mod unit;

pub struct Tex<P: AsRef<Path>> {
    pub fonts: TexFonts<P>,
    pub preamble: String,
}

impl<P: AsRef<Path>> Tex<P> {
    pub fn get_min_lines(
        &self,
        left: &[Word],
        center: &[Word],
        right: &[Word],
        table: Table,
    ) -> Result<usize, Error> {
        // Get each font command.
        let commands = [&self.fonts.left, &self.fonts.center, &self.fonts.right]
            .iter()
            .map(|f| f.command.clone())
            .collect::<Vec<String>>();
        // Get the preamble so that it can be shared between threads.
        let preamble = self.preamble.clone();
        // Get the column with the least words.
        match [left, center, right]
            .into_par_iter()
            .zip(
                [Position::Left, Position::Center, Position::Right]
                    .into_par_iter()
                    .zip(commands),
            )
            .filter_map(|(w, (p, c))| {
                if w.is_empty() {
                    None
                } else {
                    let width = table.get_width(&p);
                    Some(Self::get_num_lines(&preamble, w, &c, width).unwrap())
                }
            })
            .min()
        {
            Some(min_num_lines) => Ok(min_num_lines),
            None => Err(Error::MinNumLines),
        }
    }

    pub fn get_words(
        preamble: &str,
        font_command: &str,
        words: &[Word],
        width: Width,
        cosmic_index: usize,
        num_lines: usize,
    ) -> Result<Option<usize>, Error> {
        if words.is_empty() {
            Ok(None)
        } else {
            let mut end = cosmic_index;

            // Decrement until we have enough lines.
            while end > 0
                && Self::get_num_lines(preamble, &words[..end], font_command, width)?
                    > num_lines
            {
                end -= 1;
            }

            // Increment until we go over.
            while end < words.len()
                && Self::get_num_lines(preamble, &words[..end], font_command, width)?
                    <= num_lines
            {
                end += 1;
            }

            Ok(if end == 0 {
                None
            } else if end == words.len() {
                Some(end)
            } else {
                Some(end - 1)
            })
        }
    }

    fn get_num_lines(
        preamble: &str,
        words: &[Word],
        font_command: &str,
        width: Width,
    ) -> Result<usize, Error> {
        let (column, title) = Word::to_tex(words, font_command);

        if title || column.is_empty() {
            Ok(0)
        } else {
            // Get the preamble.
            let mut tex = preamble.to_string();

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
