use std::fmt;

use serde::{Deserialize, Serialize};

/// LaTeX units.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Unit {
    Pt,
    Mm,
    Cm,
    In,
    Em,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pt => "pt",
                Self::Mm => "mm",
                Self::Cm => "cm",
                Self::In => "in",
                Self::Em => "em",
            }
        )
    }
}
