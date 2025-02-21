use std::{fs::read_to_string, io, path::PathBuf};

use serde::Deserialize;

use crate::error::Error;

use super::raw_text::RawText;

#[derive(Deserialize)]
pub struct TextPaths {
    pub left: PathBuf,
    pub center: PathBuf,
    pub right: PathBuf,
}

impl TextPaths {
    pub fn read(&self) -> Result<RawText, Error> {
        match self.read_internal() {
            Ok(raw_text) => Ok(raw_text),
            Err(error) => Err(Error::RawText(error)),
        }
    }

    fn read_internal(&self) -> Result<RawText, io::Error> {
        let left = read_to_string(&self.left)?;
        let center = read_to_string(&self.left)?;
        let right = read_to_string(&self.left)?;
        Ok(RawText {
            left,
            center,
            right,
        })
    }
}
