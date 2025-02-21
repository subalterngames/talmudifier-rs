use std::fs::write;

use column::{tex_column::TexColumn, width::Width, Column};
use cosmic_text::FontSystem;
use error::Error;
use font::{cosmic_font::CosmicFont, tex_font::TexFont};
use page::Page;
use word::Word;

mod column;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
pub(crate) mod word;

fn main() {
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

pub fn talmudify(mut left: Column, mut center: Column, mut right: Column, page: &Page) -> Result<String, Error> {
    let mut tables = vec![];
    // First four lines.
    let table = [&mut left, &mut right].par_iter_mut().map(|c| {
        c.get_tex_column(4, Width::Half, page)
    }).collect::<Vec<Result<TexColumn, Error>>>();
    if table.iter().any(|t| t.is_err()) {
        return table.into_iter().find_map(|t| match t {
            Ok(_) => None,
            Err(error) => Some(Err(error))
        }).unwrap();
    }
    let table = table.into_iter().map(|t| t.unwrap()).collect::<Vec<TexColumn>>();
    tables.push(table);

    // Skip.
    let left_skip = left.get_tex_column(1, Width::Third, page)?;
    let right_skip = right.get_tex_column(1, Width::Third, page)?;
    tables.push(vec![left_skip, TexColumn {
        text: None,
        width: Width::Third
    }, right_skip]);

    while !left.done() && !center.done() && !right.done() {
       // let widths = Column:g
    }

    // Build the document.
    let mut tex = page.preamble.clone();
    for table in tables.iter() {
        tex.push_str(&TexColumn::get_table(table));
    }
    tex.push_str(Page::END_DOCUMENT);
    Ok(tex)
}

fn get_table(columns: &mut [&mut Column], num_lines: usize, page: &Page) -> Result<Vec<TexColumn>, Error> {
    let results = columns.iter_mut().map(|c| {
        c.get_tex_column(num_lines, Width::Half, page)
    }).collect::<Vec<Result<TexColumn, Error>>>();
    if results.iter().any(|t| t.is_err()) {
        results.into_iter().find_map(|t| match t {
            Ok(_) => None,
            Err(error) => Some(Err(error))
        }).unwrap()
    }
    else {
        Ok(results.into_iter().flat_map(|c| c).collect())
    }
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

#[cfg(test)]
mod tests {
    pub(crate) fn get_test_md() -> (String, String, String) {
        let raw = include_str!("../test_text/test.md")
            .split("\n\n")
            .collect::<Vec<&str>>();
        assert_eq!(raw.len(), 3);
        (raw[0].to_string(), raw[1].to_string(), raw[2].to_string())
    }
}
