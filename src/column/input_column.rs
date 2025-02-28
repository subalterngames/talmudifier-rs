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
