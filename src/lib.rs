//! **Convert markdown text to a PDF with Talmud-esque typesetting.**
//!
//! ```
//! use std::{fs::write, path::PathBuf, str::FromStr};
//!
//! use talmudifier::prelude::*;
//!
//! let directory = PathBuf::from_str("example_text").unwrap();
//!
//! // Load a default config file
//! let mut config = Config::default()
//!     .source_text = SourceText::Files {
//!         left: directory.join("left.md"),
//!         center: directory.join("center.md"),
//!         right: directory.join("right.md")
//! };
//!
//! // Set the source of the text.
//! config
//!
//! // Talmudify.
//! let daf = config.talmudify().unwrap();
//!
//! // Write the PDF.
//! write("out.pdf", &daf.pdf).unwrap();
//! ```

use error::Error;
use tectonic::latex_to_pdf;

mod column;
mod config;
mod error;
mod font;
mod page;
pub mod prelude;
mod word;

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

#[cfg(feature = "log")]
pub fn get_pdf(tex: &str) -> Result<Vec<u8>, Error> {
    use chrono::Utc;
    use std::{
        fs::{create_dir_all, write},
        path::PathBuf,
        str::FromStr,
    };

    const LOG_DIRECTORY: &str = "logs";

    // Create the log directory.
    create_dir_all(LOG_DIRECTORY).unwrap();

    let log_directory = PathBuf::from_str(LOG_DIRECTORY).unwrap();

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Write the tex file.
    write(log_directory.join(format!("{}.tex", &timestamp)), tex).unwrap();

    // Get the pdf.
    let pdf = get_pdf_internal(tex)?;

    // Write the PDF.
    write(log_directory.join(format!("{}.pdf", &timestamp)), &pdf).unwrap();

    Ok(pdf)
}

#[cfg(not(feature = "log"))]
pub fn get_pdf(tex: &str) -> Result<Vec<u8>, Error> {
    get_pdf_internal(tex)
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
