use std::fmt;

/// LaTeX units.
#[derive(Debug)]
pub enum Unit {
    Pt,
    Mm,
    Cm,
    In,
    Ex,
    Em,
    Mu,
    Sp,
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
                Self::Ex => "ex",
                Self::Em => "em",
                Self::Mu => "mu",
                Self::Sp => "sp",
            }
        )
    }
}
