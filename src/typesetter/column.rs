use cosmic_text::{Buffer, Shaping};

use crate::page::Page;

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

    /// Returns the index in a slice of words at which those words occupy `num_lines` number of lines.
    pub fn get_cosmic_index(&mut self, num_lines: usize, page: &Page) -> Option<usize> {
        match self {
            Self::Column { column, width } => {
                match column {
                    MaybeSpanColumn::Span(column) => {
                        let len = column.span.0.len();
                        if column.start >= len {
                            Some(0)
                        } else {
                            // Iterate through the slice.
                            for end in column.start..len - column.start {
                                // Get the width of the column in pts.
                                let column_width = page.table_width * width.column_ratio();
                                // Prepare the Cosmic buffer.
                                let mut buffer = Buffer::new(
                                    &mut column.cosmic_font.font_system,
                                    column.cosmic_font.metrics,
                                );

                                // Set the width.
                                buffer.set_size(
                                    &mut column.cosmic_font.font_system,
                                    Some(column_width),
                                    None,
                                );

                                // Get the Cosmic spans.
                                let spans = column.to_cosmic(end);
                                // Set the text.
                                buffer.set_rich_text(
                                    &mut column.cosmic_font.font_system,
                                    spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
                                    column.cosmic_font.regular.as_attrs(),
                                    Shaping::Advanced,
                                );
                                // Create lines.
                                buffer
                                    .shape_until_scroll(&mut column.cosmic_font.font_system, true);
                                // Return the number of lines.
                                let num = buffer.layout_runs().count();
                                if num > num_lines {
                                    return if end == 0 { None } else { Some(end - 1) };
                                }
                            }
                            None
                        }
                    }
                    MaybeSpanColumn::Empty => Some(0),
                }
            }
            Self::None => None,
        }
    }
}
