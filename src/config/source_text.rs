use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::raw_text::RawText;

#[derive(Deserialize, Serialize)]
pub enum SourceText {
    Text {
        left: String,
        center: String,
        right: String,
    },
    Files {
        left: PathBuf,
        center: PathBuf,
        right: PathBuf,
    },
}

impl SourceText {
    pub fn get_text(&self) -> Result<RawText, Error> {
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
    fn default() -> Self {
        Self::Files {
            left: PathBuf::from_str("left.md").unwrap(),
            center: PathBuf::from_str("center.md").unwrap(),
            right: PathBuf::from_str("right.md").unwrap(),
        }
    }
}
