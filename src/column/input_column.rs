use std::fmt::Display;

use super::Column;

/// A maybe-column containing maybe-text.
pub enum InputColumn<'t> {
    /// A column with text.
    Text(&'t mut Column),
    /// An empty column.
    Empty,
    /// Not a column.
    None,
}

impl InputColumn<'_> {
    /// If true, the column exists, though it might not have text.
    pub const fn is_column(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl Display for InputColumn<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::Text(c) => c.to_string(),
                Self::Empty => "Empty".to_string(),
                Self::None => "None".to_string(),
            }
        )
    }
}
