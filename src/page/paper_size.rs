use std::fmt;

/// The width of a page in pts.
pub const WIDTH_PTS: f32 = 614.295;

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
