use std::fmt;

use serde::{Deserialize, Serialize};

/// The size of the page.
#[derive(Default, Clone, Deserialize, Serialize)]
pub enum PaperSize {
    #[default]
    Letter,
    Legal,
    A4,
}

impl PaperSize {
    /// The page width in pts.
    pub const fn width(&self) -> f32 {
        match self {
            Self::Letter | Self::Legal => 614.295,
            Self::A4 => 597.6729,
        }
    }
}

impl fmt::Display for PaperSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::A4 => "a4paper",
                Self::Letter => "letterpaper",
                Self::Legal => "legalpaper",
            }
        )
    }
}
