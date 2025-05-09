use std::fs::write;

use tectonic::latex_to_pdf;

use crate::error::Error;

/// Render a .tex file as a .pdf
pub fn get_pdf(tex: &str) -> Result<Vec<u8>, Error> {
    // Try to generate the PDF.
    match latex_to_pdf(tex) {
        Ok(pdf) => Ok(pdf),
        Err(error) => {
            // Dump the TeX string.
            let _ = write("bad.tex", tex);
            Err(Error::Pdf(error))
        }
    }
}
