//! Generate a PDF from a .tex file. This is useful for debugging.

use std::{
    fs::{read_to_string, write},
    path::PathBuf,
};

use clap::Parser;
use talmudifier::prelude::*;
use tectonic::latex_to_pdf;

#[derive(Parser, Debug)]
struct Args {
    /// The directory that the file is in.
    #[arg(short, long, default_value = "logs")]
    directory: PathBuf,
    /// The filename.
    #[arg(short, long)]
    filename: String,
}

fn main() {
    let args = Args::parse();
    let d = DefaultTexFonts::new().unwrap();
    write(
        "out.pdf",
        latex_to_pdf(read_to_string(&args.directory.join(&args.filename)).unwrap()).unwrap(),
    )
    .unwrap();
    drop(d);
}
