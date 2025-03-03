use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(target_os = "windows")]
    #[error("Failed to create a PDF. Talmudifier cannot create PDFs on Windows.")]
    Pdf,
    #[cfg(target_os = "linux")]
    #[error("Error creating PDF: {0}")]
    Pdf(tectonic::Error),
    #[error("Error extracting text from PDF: {0}")]
    #[cfg(target_os = "linux")]
    Extract(pdf_extract::OutputError),
    #[error("Failed to get the minimum number of lines: {0}")]
    MinNumLines(String),
    #[error("Markdown parsing error: {0}")]
    Md(markdown::message::Message),
    #[error("Tried to create a table but there are no words.")]
    NoMoreWords,
    #[error("Error loading font: {0}")]
    Font(io::Error),
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
