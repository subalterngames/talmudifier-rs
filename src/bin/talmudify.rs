use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use talmudifier::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The path to a config json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used.
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
    /// The path to the output directory.
    #[arg(short, long, default_value = "out")]
    out: PathBuf,
    /// If true, write intermediate .tex and .pdf files to logs/
    #[arg(short, long)]
    log: bool,
}

fn main() {
    let args = Args::parse();

    // Get the output drectory.
    create_dir_all(&args.out).unwrap();

    // Load the talmudifier.
    let mut talmudifier = Talmudifier::new(&args.config).unwrap();

    // Enable logging.
    if args.log {
        config = config.log();
    }

    // Talmudify.
    let daf = talmudifier.talmudify().unwrap();

    // Write.
    write(args.out.join("daf.pdf"), &daf.pdf).unwrap();
    write(args.out.join("daf.tex"), &daf.tex).unwrap();
}
