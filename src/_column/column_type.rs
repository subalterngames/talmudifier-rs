/// This is used to determine which columns to include when typesetting.
pub enum ColumnType {
    /// There is an empty column.
    /// It *should* be include in the table but there's no text.
    Empty,
    /// There is no column.
    None,
    /// There is a column with text.
    /// The text is a TeX string with TeX keywords, commands, etc.
    Text(String),
}
