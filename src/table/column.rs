use super::{maybe_span_column::MaybeSpanColumn, span_column::SpanColumn, width::Width};

/// A column, possibly with text, and a width.
pub enum Column<'t> {
    /// Typeset as if there is a column, even if there isn't any text.
    Column {
        column: MaybeSpanColumn<'t>,
        width: Width,
    },
    /// Ignore this column when typesetting.
    None,
}

macro_rules! column_width {
    ($func_name:ident, $width:expr) => {
        pub fn $func_name(column: MaybeSpanColumn<'t>) -> Column<'t> {
            Column::new(column, $width)
        }
    };
}

impl<'t> Column<'t> {
    pub fn new(column: MaybeSpanColumn<'t>, width: Width) -> Self {
        match &column {
            MaybeSpanColumn::Span(span_column) => {
                // If there are no more words, ignore the column.
                if span_column.done() {
                    Self::None
                } else {
                    Self::Column { column, width }
                }
            }
            MaybeSpanColumn::Empty => Self::Column { column, width },
        }
    }

    column_width!(third, Width::Third);

    column_width!(half, Width::Half);

    column_width!(two_thirds, Width::TwoThirds);

    column_width!(one, Width::One);

    /// Returns true if there are more words to add to the page.
    pub fn done(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Try to get the underlying [`SpanColumn`].
    pub fn get_span_column(&self) -> Option<&&'t mut SpanColumn> {
        match self {
            Self::None => None,
            Self::Column { column, width: _ } => match column {
                MaybeSpanColumn::Span(column) => Some(column),
                MaybeSpanColumn::Empty => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        font::cosmic_font::CosmicFont,
        span::Span,
        table::{maybe_span_column::MaybeSpanColumn, span_column::SpanColumn, width::Width},
    };

    use super::Column;

    #[test]
    fn test_column() {
        const MD: &str = "There are so many words in this sentence!";
        let column = Column::None;
        assert!(column.done());

        let empty = MaybeSpanColumn::Empty;
        let column = Column::new(empty, Width::Half);
        assert!(!column.done());

        let span = Span::from_md(MD).unwrap();
        let mut span_column = SpanColumn::new(span, CosmicFont::default_left(), "\\font");
        span_column.start = 3;
        let full = MaybeSpanColumn::Span(&mut span_column);
        let column = Column::new(full, Width::Half);
        assert!(!column.done());

        let span = Span::from_md(MD).unwrap();
        let len = span.0.len();
        let mut span_column = SpanColumn::new(span, CosmicFont::default_left(), "\\font");
        span_column.start = len;
        let full = MaybeSpanColumn::Span(&mut span_column);
        let column = Column::new(full, Width::Half);
        assert!(column.done());
    }
}
