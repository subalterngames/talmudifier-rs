use std::fmt;

/// The position of a column on the page.
#[derive(Debug)]
pub enum Position {
    Left,
    Center,
    Right,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Left => "left",
                Self::Center => "center",
                Self::Right => "right",
            }
        )
    }
}
