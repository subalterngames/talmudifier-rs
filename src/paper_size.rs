use std::fmt;

pub enum PaperSize {
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
