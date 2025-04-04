use text_column::TextColumn;
use width_column::WidthColumn;

mod span_column;
mod text_column;
mod width_column;

pub struct Table {
    left: WidthColumn,
    center: WidthColumn,
    right: WidthColumn
}

impl Table {
    pub fn from_md(left: TextColumn, center: TextColumn, right: TextColumn) -> Self {

    }
}