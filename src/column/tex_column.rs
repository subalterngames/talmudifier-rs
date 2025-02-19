use super::Column;

/// This is used to determine which columns to include when typesetting.
pub enum TexColumn {
    /// There is an empty column.
    /// It *should* be include in the table but there's no text.
    Empty,
    /// There is no column.
    None,
    /// There is a column with text.
    Text(String),
}
