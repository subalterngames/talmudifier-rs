use super::{column::Column, maybe_span_column::MaybeSpanColumn};

pub enum ParaColumn {
    Text(String),
    Empty,
    None,
}

impl ParaColumn {
    pub fn new(column: &Column<'_>, end: Option<usize>) -> Self {
        match column {
            Column::Column { column, width: _ } => match column {
                MaybeSpanColumn::Span(column) => Self::Text(column.to_tex(end)),
                MaybeSpanColumn::Empty => Self::Empty,
            },
            Column::None => Self::None,
        }
    }

    /// Returns either an empty column or a non-column.
    pub fn new_empty(column: &Column<'_>) -> Self {
        match column {
            Column::Column {
                column: _,
                width: _,
            } => Self::Empty,
            Column::None => Self::None,
        }
    }
}
