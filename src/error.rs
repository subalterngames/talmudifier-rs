use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Page is missing a preamble")]
    NoPreamble,
    #[error("Error creating PDF: {0}")]
    Pdf(tectonic::Error),
    #[error("Error creating XDV: {0}")]
    Xdv(tectonic::Error),
    #[error("Markdown parsing error: {0}")]
    Md(markdown::message::Message),
    #[error("Cosmic font error: {0}")]
    CosmicFont(io::Error),
    #[error("Font not found: {0}")]
    NoFont(PathBuf),
    #[error("TeX fonts error: {0}")]
    TexFonts(io::Error),
    #[error("Error reading config file: {0}")]
    ConfigRead(io::Error),
    #[error("Error deserializing config file: {0}")]
    ConfigSerde(serde_json::Error),
    #[error("Error reading raw text: {0}")]
    RawText(io::Error),
    #[error("Tried to create a table with no columns")]
    NoColumns,
    #[error("Tried to read a single markdown file but found {0} paragraphs (should be 3).")]
    NumberOfParagraphs(usize),
}
