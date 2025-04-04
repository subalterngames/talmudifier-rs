use crate::{column::width::Width, span::SpanColumn};

/// A column, possibly with text, and a width.
pub enum WidthColumn<'t> {
    Column {
        column: Option<&'t mut SpanColumn>,
        width: Width,
    },
    None,
}

macro_rules! column_width {
    ($func_name:ident, $width:expr) => {
        pub fn $func_name(column: Option<&'t mut SpanColumn>) -> WidthColumn<'t> {
            WidthColumn::Column {
                column,
                width: $width,
            }
        }
    };
}

impl<'t> WidthColumn<'t> {
    column_width!(third, Width::Third);

    column_width!(half, Width::Half);

    column_width!(two_thirds, Width::TwoThirds);

    column_width!(one, Width::One);
}
