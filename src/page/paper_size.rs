use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub enum PaperSize {
    #[default]
    Letter,
    Legal,
}

impl fmt::Display for PaperSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Letter => "letterpaper",
                Self::Legal => "legalpaper",
            }
        )
    }
}
