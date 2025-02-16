use std::{io, path::Path};

use pdf_extract::extract_text_from_mem;
use tectonic::latex_to_pdf;
use tempdir::TempDir;

use crate::{font::tex_font::TexFont, word::Word};

use super::{error::Error, width::Width, ColumnMaker};

pub struct Tex<P: AsRef<Path>> {
    pub font: TexFont<P>,
    pub column_width: Width,
}

impl<P: AsRef<Path>> ColumnMaker for Tex<P> {
    fn get_num_lines(&mut self, words: &[Word]) -> Result<usize, Error> {
        let (tex, title) = Word::to_tex(words, &self.font.command);

        // TODO Convert to a paracol.

        if title || tex.is_empty() {
            Ok(0)
        } else {
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
