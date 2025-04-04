
use crate::column::width::Width;

use super::text_column::TextColumn;

/// A maybe-column and a width.
pub enum WidthColumn {
    Column { column: TextColumn, width: Width },
    None
}