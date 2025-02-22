use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use clap::{Arg, Command};
use config::Config;
use error::Error;
use talmudifier::Talmudifier;

mod column;
mod config;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
mod talmudifier;
pub(crate) mod word;

fn main() {
    let args = Command::new("talmudifier")
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .args([
            Arg::new("config")
            .short('c')
            .required(false)
            .help("The absolute path to a config json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used."),
            Arg::new("out")
                .short('o')
                .default_value("out.pdf")
                .help("The absolute path to the output file."),
        ]).get_matches();
    // Get the fonts.
    let path = args.get_one::<PathBuf>("config").unwrap();

    // Get the output drectory.
    let output_path = args.get_one::<PathBuf>("out").unwrap();
    create_dir_all(output_path.parent().unwrap()).unwrap();

    // Load the config file.
    let config = Config::new(path).unwrap();

    // Talmudify.
    let mut talmudifier = Into::<Result<Talmudifier, Error>>::into(config).unwrap();
    let daf = talmudifier.talmudify().unwrap();

    // Write.
    write(output_path, daf).unwrap();
}

#[macro_export]
macro_rules! tex {
    ($command:expr, $($value:expr),+) => {
        {
            let mut t = format!("\\{}", &$command);
            $(
                t.push_str(&format!("{{{}}}", &$value));
            )+
            t
        }
    };
}

#[macro_export]
macro_rules! text_arg {
    ($column:expr) => {
        Arg::new(concat!($column, "-text")).help(format!(
            "The absolute file path to the {} markdown text.",
            $column
        ))
    };
}

#[cfg(test)]
mod tests {
    pub(crate) fn get_test_md() -> (String, String, String) {
        let raw = include_str!("../test_text/test.md")
            .split("\n\n")
            .collect::<Vec<&str>>();
        assert_eq!(raw.len(), 3);
        (raw[0].to_string(), raw[1].to_string(), raw[2].to_string())
    }
}
