use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use clap::Parser;
use talmudifier::{latex_to_xdv, prelude::*};
use tectonic::latex_to_pdf;

/// Generate a PDF from a .tex file. This is useful for debugging.
///
/// 1. Find a PDF + .tex file that looks weird
/// 2. Manually edit the .tex file
/// 3. Run `textest`
/// 4. Rinse, repeat
#[derive(Parser, Debug)]
struct Args {
    /// The directory that the file is in.
    #[arg(short, long, default_value = "logs")]
    directory: PathBuf,
    /// The filename.
    #[arg(short, long)]
    filename: String,
    /// If included, extract text.
    #[arg(short, long)]
    text: bool,
}

fn main() {
    let args = Args::parse();
    let d = DefaultTexFonts::new().unwrap();
    let tex = read_to_string(&args.directory.join(&args.filename)).unwrap();

    let pdf = latex_to_pdf(read_to_string(&args.directory.join(&args.filename)).unwrap()).unwrap();

    // Extract text.
    if args.text {
        let text = extract_text_from_mem(&pdf).unwrap();
        write("extracted_text.txt", text).unwrap();
    }

    // Write the PDF.
    write("out.pdf", pdf).unwrap();

    // Remove the temporary fonts.
    drop(d);
}
