use std::fmt::{Display, Formatter};

/// The position of a column on the page.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Position {
    Left,
    Center,
    Right,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        };
        write!(f, "{s}")
    }
}

pub const POSITIONS: [Position; 3] = [Position::Left, Position::Center, Position::Right];
