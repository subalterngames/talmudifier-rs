use std::path::Path;

use column::{input_column::InputColumn, tex_column::TexColumn, Column};
use config::Config;
use error::Error;
use page::Page;
use word::Word;

mod column;
pub mod config;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
pub(crate) mod word;

pub struct Talmudifier {
    left: Column,
    center: Column,
    right: Column,
    page: Page,
    title: Option<String>,
}

impl Talmudifier {
    /// Load from a config.json file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Into::<Result<Talmudifier, Error>>::into(Config::new(path)?)
    }

    pub fn talmudify(&mut self) -> Result<String, Error> {
        // First four lines.
        let mut tables = vec![Column::get_tex_table(
            &mut InputColumn::Text(&mut self.left),
            &mut InputColumn::None,
            &mut InputColumn::Text(&mut self.right),
            4,
            &self.page,
        )?];
        // Skip.
        tables.push(Column::get_tex_table(
            &mut InputColumn::Text(&mut self.left),
            &mut InputColumn::Empty,
            &mut InputColumn::Text(&mut self.right),
            1,
            &self.page,
        )?);

        while !self.left.done() && !self.center.done() && !self.right.done() {
            // Get the columns that are and are not done.
            let left_optional = Self::get_optional_column(&self.left);
            let center_optional = Self::get_optional_column(&self.center);
            let right_optional = Self::get_optional_column(&self.right);

            // Get the minimum number of lines.
            let num_lines = Column::get_min_num_lines(
                left_optional,
                center_optional,
                right_optional,
                &self.page,
            )?;

            // Get all available columns.
            let mut left = Self::get_input_column(&mut self.left);
            let mut center = Self::get_input_column(&mut self.center);
            let mut right = Self::get_input_column(&mut self.right);

            // Create the table.
            tables.push(Column::get_tex_table(
                &mut left,
                &mut center,
                &mut right,
                num_lines,
                &self.page,
            )?);

            // Skip to the next table.
            left = Self::get_input_column_skip(left);
            center = Self::get_input_column_skip(center);
            right = Self::get_input_column_skip(right);

            // Create the table.
            tables.push(Column::get_tex_table(
                &mut left,
                &mut center,
                &mut right,
                1,
                &self.page,
            )?);
        }

        // Build the document.
        let mut tex = self.page.preamble.clone();
        // Add the title.
        if let Some(title) = &self.title {
            tex.push_str(&crate::tex!("chapter", crate::tex!("daftitle", title)));
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

    fn get_input_column(column: &mut Column) -> InputColumn<'_> {
        if column.done() {
            InputColumn::None
        } else {
            InputColumn::Text(column)
        }
    }

    fn get_input_column_skip(column: InputColumn<'_>) -> InputColumn<'_> {
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
}

impl From<Config> for Result<Talmudifier, Error> {
    fn from(value: Config) -> Self {
        // Get the TeX fonts.
        let tex_fonts = value.get_tex_fonts()?;
        // Clone the page.
        let mut page = value.page.clone();
        // Set the preamble.
        page.set_preamble(&tex_fonts);

        // Get the raw text.
        let raw_text = value.text_paths.read()?;

        // Get the words.
        let left_words = Word::from_md(&raw_text.left)?;
        let center_words = Word::from_md(&raw_text.center)?;
        let right_words = Word::from_md(&raw_text.right)?;

        // Get the cosmic fonts.
        let (left_cosmic, center_cosmic, right_cosmic) = value.get_cosmic_fonts()?;

        let left = Column::new(left_words, left_cosmic, &tex_fonts.left.command);
        let center = Column::new(center_words, center_cosmic, &tex_fonts.center.command);
        let right = Column::new(right_words, right_cosmic, &tex_fonts.right.command);

        Ok(Talmudifier {
            left,
            center,
            right,
            page,
            title: value.title,
        })
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
    use tectonic::latex_to_pdf;

    pub(crate) fn get_test_md() -> (String, String, String) {
        let raw = include_str!("../test_text/test.md")
            .split("\n\n")
            .collect::<Vec<&str>>();
        assert_eq!(raw.len(), 3);
        (raw[0].to_string(), raw[1].to_string(), raw[2].to_string())
    }

    #[test]
    fn test_tex() {
        for (tex, path) in [
            include_str!("../test_text/hello_world.tex"),
            include_str!("../test_text/minimal_daf.tex"),
            include_str!("../test_text/paracol.tex"),
            include_str!("../test_text/daf.tex"),
        ]
        .iter()
        .zip(["hello_world", "minimal_daf", "paracol", "daf"])
        {
            if let Err(error) = latex_to_pdf(tex.replace("\r", "")) {
                panic!("Tex error: {} {}", error, path)
            }
        }
    }
}
