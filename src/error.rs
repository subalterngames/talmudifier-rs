use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(not(target_os = "windows"))]
    #[error("Error creating PDF: {0}")]
    Pdf(tectonic::Error),
    #[error("Error extracting text from PDF: {0}")]
    #[cfg(not(target_os = "windows"))]
    Extract(pdf_extract::OutputError),
    #[error("Failed to get the minimum number of lines: {0}")]
    MinNumLines(String),
    #[error("Failed to get the minimum number of lines")]
    NoMinNumLines,
    #[error("Markdown parsing error: {0}")]
    Md(markdown::message::Message),
    #[error("Tried to create a table but there are no words.")]
    NoMoreWords,
}
