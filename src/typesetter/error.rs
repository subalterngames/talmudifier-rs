use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Tectonic error: {0}")]
    Tectonic(tectonic::Error),
    #[error("Extraction error: {0}")]
    Extraction(pdf_extract::OutputError),
}
