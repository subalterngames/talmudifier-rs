use super::{maybe_span_column::MaybeSpanColumn, width::Width};

/// A column, possibly with text, and a width.
pub enum Column<'t> {
    Column {
        column: MaybeSpanColumn<'t>,
        width: Width,
    },
    None,
}

macro_rules! column_width {
    ($func_name:ident, $width:expr) => {
        pub fn $func_name(column: MaybeSpanColumn<'t>) -> Column<'t> {
            Column::Column {
                column,
                width: $width,
            }
        }
    };
}

impl<'t> Column<'t> {
    column_width!(third, Width::Third);

    column_width!(half, Width::Half);

    column_width!(two_thirds, Width::TwoThirds);

    column_width!(one, Width::One);
}
