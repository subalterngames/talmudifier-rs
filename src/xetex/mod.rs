use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
    str::FromStr,
};

use chrono::Utc;

mod pdf;
mod xdv;

pub use pdf::get_pdf;
pub use xdv::get_num_lines;

#[cfg(feature = "textest")]
pub use xdv::latex_to_xdv;

/// Write a .tex file to logs/
pub fn log_tex(tex: &str) {
    const LOG_DIRECTORY: &str = "logs";

    let log_directory = PathBuf::from_str(LOG_DIRECTORY).unwrap();
    // Create the log directory.
    create_dir_all(LOG_DIRECTORY).unwrap();

    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();

    // Write the tex file.
    write(log_directory.join(format!("{}.tex", &timestamp)), tex).unwrap();
}

