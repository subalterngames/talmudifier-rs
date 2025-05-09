use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use clap::Parser;
use talmudifier::{xetex::latex_to_xdv, DefaultTexFonts};
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
    /// If included, create an .xdv file instead of a .pdf
    #[arg(short, long)]
    xdv: bool,
}

fn main() {
    let args = Args::parse();
    let d = DefaultTexFonts::new().unwrap();
    let latex = read_to_string(&args.directory.join(&args.filename)).unwrap();

    let (path, data) = if args.xdv {
        ("out.xdv", latex_to_xdv(&latex).unwrap())
    } else {
        ("out.pdf", latex_to_pdf(&latex).unwrap())
    };
    // Write the PDF.
    write(path, data).unwrap();

    // Remove the temporary fonts.
    drop(d);
}
