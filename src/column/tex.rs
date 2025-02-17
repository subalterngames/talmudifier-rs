use std::{io, path::Path};

use pdf_extract::extract_text_from_mem;
use tectonic::latex_to_pdf;
use tempdir::TempDir;

use crate::{
    font::tex_font::TexFont,
    tex::{column_type::ColumnType, page::Page, Table},
    word::Word,
};

use super::{error::Error, width::Width, ColumnMaker};

pub struct Tex<'t, P: AsRef<Path>> {
    pub font: &'t TexFont<P>,
    pub width: Width,
    pub preamble: &'t str,
}

impl<'t, P: AsRef<Path>> Tex<'t, P> {
    pub fn get_tex(&self, column: String) -> String {
        // Get the preamble.
        let mut tex = self.preamble.to_string();

        // Get a table.
        tex.push_str(&match self.width {
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
        tex
    }
}

impl<'t, P: AsRef<Path>> ColumnMaker for Tex<'t, P> {
    fn get_num_lines(&mut self, words: &[Word]) -> Result<usize, Error> {
        let (column, title) = Word::to_tex(words, &self.font.command);

        if title || column.is_empty() {
            Ok(0)
        } else {
            let tex = self.get_tex(column);
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
