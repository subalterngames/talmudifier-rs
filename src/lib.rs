#![doc = include_str!("../doc/README_rs.md")]
//!
#![cfg_attr(all(),
doc = embed_doc_image::embed_image!("daf", "images/daf.jpg"),
doc = embed_doc_image::embed_image!("four_rows", "images/four_rows.jpg"),
doc = embed_doc_image::embed_image!("center", "images/center.jpg"))]

use std::{
    fs::{create_dir_all, read, write},
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::Utc;
use error::Error;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tectonic::latex_to_pdf;
use text::{Daf, SourceText};

use crate::{
    font::fonts::Fonts,
    page::Page,
    span::Span,
    table::{maybe_span_column::MaybeSpanColumn, span_column::SpanColumn, OptionalColumn, Table},
};

mod error;
mod font;
mod page;
pub mod prelude;
mod span;
mod table;
mod text;

/// Short hand for simple TeX commands.
/// Example input: `tex!("begin", "document")`
/// Example output: `\begin{document}`
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

/// Generate a Talmud-like page.
#[cfg_attr(feature = "default-fonts", derive(Default))]
#[derive(Deserialize, Serialize)]
pub struct Talmudifier {
    /// The size of the page, margins, etc.
    page: Page,
    /// The fonts per column.
    #[cfg_attr(feature = "default-fonts", serde(default = "Fonts::default"))]
    fonts: Fonts,
    /// Raw markdown text that will be talmudified.
    source_text: SourceText,
    /// If not None, the title will be at the top of the page.
    title: Option<String>,
    /// If true, logging is enabled.
    log: bool,
}

impl Talmudifier {
    /// Load config data from a file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match read(path) {
            Ok(text) => match from_slice(&text) {
                Ok(config) => Ok(config),
                Err(error) => Err(Error::ConfigSerde(error)),
            },
            Err(error) => Err(Error::ConfigRead(error)),
        }
    }

    /// Set the page layout parameters.
    pub fn page(mut self, page: Page) -> Self {
        self.page = page;
        self
    }

    /// Set the fonts.
    pub fn fonts(mut self, fonts: Fonts) -> Self {
        self.fonts = fonts;
        self
    }

    /// Set the source Markdown text.
    pub fn source_text(mut self, source_text: SourceText) -> Self {
        self.source_text = source_text;
        self
    }

    /// Set the title text. By default, there is no title.
    pub fn title<S: ToString>(mut self, title: S) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Enable logging.
    /// This will generate intermediary .pdf, .tex, and .txt files per iteration.
    /// The .txt files include the text extracted from the .pdf
    /// This function is useful for debugging because you can identify where a typesetting error occurred.
    /// However, this function will make [`self.talmudifiy()`] run slower.
    pub fn log(mut self) -> Self {
        self.log = true;
        self
    }

    /// Convert raw markdown text into a Talmud page.
    /// This can take a while (on the other of minutes).
    /// Be patient!
    ///
    /// Returns a `Daf` containing the TeX string and the PDF.
    pub fn talmudify(&self) -> Result<Daf, Error> {
        // Get the TeX fonts.
        let tex_fonts = self.fonts.tex_fonts()?;

        // Clone the page.
        let mut page = self.page.clone();

        // Set the preamble.
        page.set_preamble(&tex_fonts);

        // Get the raw text.
        let raw_text = self.source_text.get_text()?;

        // Get the words.
        let left_span = Span::from_md(&raw_text.left)?;
        let center_span = Span::from_md(&raw_text.center)?;
        let right_span = Span::from_md(&raw_text.right)?;

        // Get the cosmic fonts.
        let cosmic_fonts = self.fonts.cosmic_fonts(&self.page.font_metrics)?;

        // Get the columns.
        let mut left = SpanColumn::new(left_span, cosmic_fonts.left, &tex_fonts.left.command);
        let mut center =
            SpanColumn::new(center_span, cosmic_fonts.center, &tex_fonts.center.command);
        let mut right = SpanColumn::new(right_span, cosmic_fonts.right, &tex_fonts.right.command);

        let mut tables = vec![];

        // First four lines.
        let mut table = Table::new(
            Some(MaybeSpanColumn::Span(&mut left)),
            None,
            Some(MaybeSpanColumn::Span(&mut right)),
            &self.page,
            self.log,
        );

        let mut done = false;

        match table.get_tex_table(None, 4)? {
            Some(table) => {
                tables.push(table);
            }
            None => done = true,
        }

        if !done {
            done = table.done();
        }

        // Skip.
        if !done {
            table = Table::new(
                Some(MaybeSpanColumn::Span(&mut left)),
                Some(MaybeSpanColumn::Empty),
                Some(MaybeSpanColumn::Span(&mut right)),
                &self.page,
                self.log,
            );
            match table.get_tex_table(None, 1)? {
                Some(table) => tables.push(table),
                None => done = true,
            }
        }

        if !done {
            done = table.done();
        }

        // Title.
        if !done {
            if let Some(title) = &self.title {
                table = Table::new(
                    Some(MaybeSpanColumn::Span(&mut left)),
                    Some(MaybeSpanColumn::Empty),
                    Some(MaybeSpanColumn::Span(&mut right)),
                    &self.page,
                    self.log,
                );
                match table.get_title_table(title)? {
                    Some(table) => tables.push(table),
                    None => done = true,
                }
            }
        }

        if !done {
            done = table.done();
        }

        while !done {
            // Decide which columns to use.
            let left_column = Self::get_column(&mut left);
            let center_column = Self::get_column(&mut center);
            let right_column = Self::get_column(&mut right);

            // Get the columns already done.
            let was_done = [&left_column, &center_column, &right_column]
                .iter()
                .map(|c| c.is_none())
                .collect::<Vec<bool>>();

            // Create a table.
            table = Table::new(
                left_column,
                center_column,
                right_column,
                &self.page,
                self.log,
            );

            // Get the minimum number of lines.
            let (num_lines, position) = table.get_min_num_lines()?;
            match num_lines {
                Some(num_lines) =>
                // Generate the table.
                {
                    match table.get_tex_table(Some(position), num_lines)? {
                        Some(table) => tables.push(table),
                        None => done = true,
                    }
                }
                // There is only one column. Stop here.
                None => {
                    tables.push(table.get_tex_table_one_column());
                    done = true;
                }
            }

            if !done {
                done = table.done();
            }

            // Skip.
            if !done {
                table = Table::new(
                    Self::get_skip_column(&mut left, was_done[0]),
                    Self::get_skip_column(&mut center, was_done[1]),
                    Self::get_skip_column(&mut right, was_done[2]),
                    &self.page,
                    self.log,
                );
                match table.get_tex_table(None, 1)? {
                    Some(table) => tables.push(table),
                    None => done = true,
                }
            }

            // Re-check if we're done.
            if !done {
                done = table.done();
            }
        }

        // Build the document.
        let mut tex = self.page.preamble.clone();

        // Add the tables.
        tex.push_str(&tables.join("\n"));
        // End the document.
        tex.push_str(Page::END_DOCUMENT);

        // Generate the final PDF.
        let pdf = get_pdf(&tex, self.log)?;
        Ok(Daf { tex, pdf })
    }

    fn get_column(span_column: &mut SpanColumn) -> OptionalColumn<'_> {
        if span_column.done() {
            None
        } else {
            Some(MaybeSpanColumn::Span(span_column))
        }
    }

    fn get_skip_column(span_column: &mut SpanColumn, was_done: bool) -> OptionalColumn<'_> {
        if span_column.done() {
            if was_done {
                None
            } else {
                Some(MaybeSpanColumn::Empty)
            }
        } else {
            Some(MaybeSpanColumn::Span(span_column))
        }
    }
}

pub(crate) fn get_pdf(tex: &str, log: bool) -> Result<Vec<u8>, Error> {
    const LOG_DIRECTORY: &str = "logs";

    let log_directory = PathBuf::from_str(LOG_DIRECTORY).unwrap();
    if log {
        // Create the log directory.
        create_dir_all(LOG_DIRECTORY).unwrap();
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    Ok(if log {
        // Write the tex file.
        write(log_directory.join(format!("{}.tex", &timestamp)), tex).unwrap();

        // Get the pdf.
        let pdf = get_pdf_internal(tex)?;

        // Write the PDF.
        write(log_directory.join(format!("{}.pdf", &timestamp)), &pdf).unwrap();

        pdf
    } else {
        get_pdf_internal(tex)?
    })
}

fn get_pdf_internal(tex: &str) -> Result<Vec<u8>, Error> {
    // Try to generate the PDF.
    match latex_to_pdf(tex) {
        Ok(pdf) => Ok(pdf),
        Err(error) => {
            // Dump the TeX string.
            let _ = write("bad.tex", tex);
            Err(Error::Pdf(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_slice;

    use crate::{get_pdf, Talmudifier};

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
            if let Err(error) = get_pdf(&tex.replace("\r", ""), false) {
                panic!("Tex error: {} {}", error, path)
            }
        }
    }

    #[test]
    fn from_example_json() {
        from_slice::<Talmudifier>(include_bytes!("../example_talmudifier.json")).unwrap();
    }
}
