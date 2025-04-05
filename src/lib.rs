//! **Convert markdown text to a PDF with Talmud-esque typesetting.**
//!
//! ```
//! use std::{fs::write, path::PathBuf, str::FromStr};
//!
//! use talmudifier::prelude::*;
//!
//! let directory = PathBuf::from_str("example_text").unwrap();
//!
//! // Load a default config file.
//! let mut config = Config::default()
//!     // Enable logging.
//!     .log()
//!     // Set the source text as three Markdown files.
//!     .source_text(SourceText::Files {
//!         left: directory.join("left.md"),
//!         center: directory.join("center.md"),
//!         right: directory.join("right.md")
//! });
//!
//! // Talmudify.
//! let daf = config.talmudify().unwrap();
//!
//! // Write the PDF.
//! write("out.pdf", &daf.pdf).unwrap();
//! ```

use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
    str::FromStr,
};

use chrono::Utc;
use error::Error;
use pdf_extract::extract_text_from_mem;
use tectonic::latex_to_pdf;

mod config;
mod error;
mod font;
mod page;
pub mod prelude;
mod span;
mod table;

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

pub(crate) fn get_pdf(
    tex: &str,
    log: bool,
    count_lines: bool,
) -> Result<(Vec<u8>, Option<usize>), Error> {
    const LOG_DIRECTORY: &str = "logs";

    let log_directory = PathBuf::from_str(LOG_DIRECTORY).unwrap();
    if log {
        // Create the log directory.
        create_dir_all(LOG_DIRECTORY).unwrap();
    }

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    let pdf = if log {
        // Write the tex file.
        write(log_directory.join(format!("{}.tex", &timestamp)), tex).unwrap();

        // Get the pdf.
        let pdf = get_pdf_internal(tex)?;

        // Write the PDF.
        write(log_directory.join(format!("{}.pdf", &timestamp)), &pdf).unwrap();

        pdf
    } else {
        get_pdf_internal(tex)?
    };

    if count_lines {
        match extract_text_from_mem(&pdf) {
            Ok(text) => {
                // Log the extracted text.
                if log {
                    write(log_directory.join(format!("{}.txt", &timestamp)), &text).unwrap();
                }
                let num_lines = Some(text.split('\n').filter(|s| !s.is_empty()).count());
                // Return the number of lines.
                Ok((pdf, num_lines))
            }
            Err(error) => Err(Error::Extract(error)),
        }
    } else {
        Ok((pdf, None))
    }
}

fn get_pdf_internal(tex: &str) -> Result<Vec<u8>, Error> {
    // Try to generate the PDF.
    match latex_to_pdf(tex) {
        Ok(pdf) => Ok(pdf),
        Err(error) => Err(Error::Pdf(error)),
    }
}

#[cfg(test)]
mod tests {
    use crate::get_pdf;

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
            if let Err(error) = get_pdf(&tex.replace("\r", ""), false, false) {
                panic!("Tex error: {} {}", error, path)
            }
        }
    }
}
