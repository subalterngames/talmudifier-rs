use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::raw_text::RawText;

/// The source for the raw markdown text.
#[derive(Deserialize, Serialize)]
pub enum SourceText {
    /// Three markdown strings.
    Text {
        left: String,
        center: String,
        right: String,
    },
    /// Three paths to markdown files.
    Files {
        left: PathBuf,
        center: PathBuf,
        right: PathBuf,
    },
    /// A single file of exactly three markdown paragraphs.
    File(PathBuf),
}

impl SourceText {
    pub(crate) fn get_text(&self) -> Result<RawText, Error> {
        match self {
            Self::Text {
                left,
                center,
                right,
            } => Ok(RawText {
                left: left.clone(),
                center: center.clone(),
                right: right.clone(),
            }),
            Self::Files {
                left,
                center,
                right,
            } => match Self::read_internal(left, center, right) {
                Ok(raw_text) => Ok(raw_text),
                Err(error) => Err(Error::RawText(error)),
            },
            Self::File(path) => match read_to_string(path) {
                Ok(md) => {
                    let md = md.split("\n\n").collect::<Vec<&str>>();
                    if md.len() == 3 {
                        Ok(RawText {
                            left: md[0].to_string(),
                            center: md[1].to_string(),
                            right: md[2].to_string(),
                        })
                    } else {
                        Err(Error::NumberOfParagraphs(md.len()))
                    }
                }
                Err(error) => Err(Error::RawText(error)),
            },
        }
    }

    fn read_internal(left: &Path, center: &Path, right: &Path) -> Result<RawText, io::Error> {
        let left = read_to_string(left)?;
        let center = read_to_string(center)?;
        let right = read_to_string(right)?;
        Ok(RawText {
            left,
            center,
            right,
        })
    }
}

impl Default for SourceText {
    /// Try to read text from three files: `left.md`, `center.md`, and `right.md`
    /// These files must be in the same folder as the working directory.
    fn default() -> Self {
        Self::Files {
            left: PathBuf::from_str("left.md").unwrap(),
            center: PathBuf::from_str("center.md").unwrap(),
            right: PathBuf::from_str("right.md").unwrap(),
        }
    }
}
