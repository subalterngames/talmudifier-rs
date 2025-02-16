use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error creating PDF: {0}")]
    Pdf(tectonic::Error),
    #[error("Error extracting text from PDF: {0}")]
    Extract(pdf_extract::OutputError),
}
