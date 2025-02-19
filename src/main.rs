use std::fs::write;

use column::{cosmic::Cosmic, tex::Tex, width::Width, ColumnMaker};
use cosmic_text::FontSystem;
use error::Error;
use font::{cosmic_font::CosmicFont, tex_font::TexFont};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tectonic::latex_to_pdf;
use tex::{column_type::ColumnType, page::Page, table::Table, Tex};
use typeset_table::TypesetTable;
use word::Word;

mod column;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
pub(crate) mod tex;
mod typeset_table;
pub(crate) mod word;

fn main() {
    let md = include_str!("test.md");
    let page = Page::default();
    let (left, center, right) = TexFont::default_fonts().unwrap();
    let fonts = [&left, &center, &right];
    let preamble = page.get_preamble(fonts);

    let width = Width::Half;

    let mut font_system = FontSystem::new();
    let cosmic_font = CosmicFont::default_left(&mut font_system);
    let table_width = page.get_table_width();
    let mut cosmic = Cosmic::new(&cosmic_font, width, table_width, &mut font_system);

    let words = Word::from_md(md).unwrap();

    let index = cosmic.get_words(&words, 4).unwrap().unwrap();

    /*
    let mut tex = Tex {
        preamble: &preamble,
        font: &left,
        width,
    };

    // TODO make the tex decrement.

    let index = tex.get_words(&words, 4).unwrap().unwrap();

    let (column, _) = Word::to_tex(&words[..index], &left.command);
    let tex = tex.get_tex(column);

    let pdf = latex_to_pdf(&tex).unwrap();
    write("out.pdf", pdf).unwrap();*/
}

pub fn get_table<'t>(
    tex: &'t Tex,
    left: &'t [Word],
    center: &'t [Word],
    right: &'t [Word],
    num_lines: Option<usize>,
) -> Result<TypesetTable<'t>, Error> {
    // Derive the table from which columns still have words.
    let table = match (!left.is_empty(), !center.is_empty(), !right.is_empty()) {
        (true, true, true) => Table::Three,
        (true, false, false) | (false, true, false) | (false, false, true) => Table::One,
        (true, true, false) => Table::LeftCenter,
        (true, false, true) => Table::LeftRight,
        (false, true, true) => Table::CenterRight,
        (false, false, false) => {
            return Err(Error::NoMoreWords);
        }
    };
    // Get the target number of lines.
    let num_lines = match num_lines {
        // Use a hardcoded number of lines.
        Some(num_lines) => num_lines,
        // Get the minimum number of lines.
        None => tex.get_min_lines(left, center, right, table)?,
    };
    let end_indices = [left, center, right].into_par_iter().map(|words| {
        // TODO get the cosmic index.
    });
}

#[macro_export]
macro_rules! tex {
    ($command:expr, $($value:expr),+) => {
        {
            let mut t = format!("\\{}", &$command);
            $(
                t.push_str(&format!("{{{}}}", &$value));
            )+
            t
        }
    };
}
