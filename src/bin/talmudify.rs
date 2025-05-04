use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use clap::Parser;
use talmudifier::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, arg_required_else_help = true)]
struct Args {
    /// The path to a talmudifier json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used.
    #[arg(short, long, default_value = "talmudifier.json")]
    talmudifier: PathBuf,
    /// The path to the output directory.
    #[arg(short, long, default_value = "out")]
    out: PathBuf,
    /// If included, write intermediate .tex and .pdf files to logs/. This is useful for debugging but slow.
    #[arg(short, long)]
    log: bool,
}

fn main() {
    let args = Args::parse();

    // Get the output drectory.
    create_dir_all(&args.out).expect(&format!("Failed to create directory: {:?}", &args.out));

    // Load the talmudifier.
    let mut talmudifier = Talmudifier::new(&args.talmudifier).unwrap();

    // Enable logging.
    if args.log {
        talmudifier = talmudifier.log();
    }

    // Talmudify.
    let daf = talmudifier.talmudify().unwrap();

    // Write.
    write(args.out.join("daf.pdf"), &daf.pdf).unwrap();
    write(args.out.join("daf.tex"), &daf.tex).unwrap();
}
