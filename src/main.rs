use std::{fs::write, path::PathBuf};

use clap::{Arg, Command};
use column::{input_column::InputColumn, tex_column::TexColumn, width::Width, Column};
use config::Config;
use cosmic_text::FontSystem;
use error::Error;
use font::{cosmic_font::CosmicFont, tex_font::TexFont, tex_fonts::TexFonts};
use page::Page;
use word::Word;

mod column;
mod config;
mod daf;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
pub(crate) mod word;

fn main() {
    let args = Command::new("talmudifier")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .args([
            Arg::new("-config")
            .short('c')
            .required(false)
            .help("The absolute path to a config json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used."),
            Arg::new("output-path")
                .short('o')
                .default_value("out.pdf")
                .help("The absolute path to the output file."),
        ]).get_matches();
    // Get the fonts.
    let path = args.get_one::<PathBuf>("config").unwrap();
    let config = Config::new(path).unwrap();

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

pub fn talmudify(
    mut left: Column,
    mut center: Column,
    mut right: Column,
    title: Option<String>,
    page: &Page,
) -> Result<String, Error> {
    // First four lines.
    let mut tables = vec![Column::get_tex_table(
        &mut InputColumn::Text(&mut left),
        &mut InputColumn::None,
        &mut InputColumn::Text(&mut right),
        4,
        page,
    )?];
    // Skip.
    tables.push(Column::get_tex_table(
        &mut InputColumn::Text(&mut left),
        &mut InputColumn::Empty,
        &mut InputColumn::Text(&mut right),
        1,
        page,
    )?);

    while !left.done() && !center.done() && !right.done() {
        // Get the columns that are and are not done.
        let left_optional = get_optional_column(&left);
        let center_optional = get_optional_column(&center);
        let right_optional = get_optional_column(&right);

        // Get the minimum number of lines.
        let num_lines =
            Column::get_min_num_lines(left_optional, center_optional, right_optional, page)?;

        // Get all available columns.
        let mut left = get_input_column(&mut left);
        let mut center = get_input_column(&mut center);
        let mut right = get_input_column(&mut right);

        // Create the table.
        tables.push(Column::get_tex_table(
            &mut left,
            &mut center,
            &mut right,
            num_lines,
            page,
        )?);

        // Skip to the next table.
        left = get_input_column_skip(left);
        center = get_input_column_skip(center);
        right = get_input_column_skip(right);

        // Create the table.
        tables.push(Column::get_tex_table(
            &mut left,
            &mut center,
            &mut right,
            1,
            page,
        )?);
    }

    // Build the document.
    let mut tex = page.preamble.clone();
    // Add the title.
    if let Some(title) = title {
        tex.push_str(&tex!("chapter", tex!("daftitle", title)));
        tex.push('\n');
    }
    for table in tables.iter() {
        tex.push_str(&TexColumn::get_table(table));
    }
    tex.push_str(Page::END_DOCUMENT);
    Ok(tex)
}

fn get_optional_column(column: &Column) -> Option<&Column> {
    if column.done() {
        None
    } else {
        Some(column)
    }
}

fn get_input_column<'t>(column: &'t mut Column) -> InputColumn<'t> {
    if column.done() {
        InputColumn::None
    } else {
        InputColumn::Text(column)
    }
}

fn get_input_column_skip<'t>(column: InputColumn<'t>) -> InputColumn<'t> {
    match column {
        InputColumn::None => InputColumn::None,
        InputColumn::Empty => InputColumn::Empty,
        InputColumn::Text(text) => {
            // Skip.
            if text.done() {
                InputColumn::Empty
            }
            // Include.
            else {
                InputColumn::Text(text)
            }
        }
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

#[macro_export]
macro_rules! text_arg {
    ($column:expr) => {
        Arg::new(concat!($column, "-text")).help(format!(
            "The absolute file path to the {} markdown text.",
            $column
        ))
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
