//! **Convert markdown text to a PDF with Talmud-esque typesetting.**
//!
//! ```
//! use std::{fs::write, path::PathBuf, str::FromStr};
//!
//! use talmudifier::prelude::*;
//!
//! let directory = PathBuf::from_str("example_text").unwrap();
//!
//! let mut config = Config::default();
//!
//! // Set the source of the text.
//! config.source_text = SourceText::Files {
//!     left: directory.join("left.md"),
//!     center: directory.join("center.md"),
//!     right: directory.join("right.md")
//! };
//!
//! // Talmudify.
//! let daf = config.talmudify().unwrap();
//!
//! // Write the PDF.
//! write("out.pdf", &daf.pdf).unwrap();
//! ```

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
