use std::{
    path::Path,
    thread::{spawn, JoinHandle},
};

use column_type::ColumnType;
use error::Error;
use page::Page;
use pdf_extract::extract_text_from_mem;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use table::Table;
use tectonic::latex_to_pdf;

use crate::{column::width::Width, error::Error, font::tex_fonts::TexFonts, word::Word};

pub(crate) mod column_type;
pub(crate) mod table;

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
}
