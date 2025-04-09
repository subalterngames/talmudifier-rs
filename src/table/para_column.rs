use super::{column::Column, maybe_span_column::MaybeSpanColumn};

/// A column with some TeX.
#[derive(Default, Clone)]
pub enum ParaColumn {
    Text(String),
    Empty,
    #[default]
    None,
}

impl ParaColumn {
    pub fn new(column: &Column<'_>, end: Option<usize>) -> Self {
        match column {
            Column::Column { column, width: _ } => match column {
                MaybeSpanColumn::Span(column) => {
                    // Set the end index.
                    let end = match end {
                        Some(end) => end,
                        None => column.span.0.len(),
                    };
                    if column.start >= end {
                        Self::None
                    } else {
                        Self::Text(column.to_tex(Some(end)))
                    }
                }
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

#[cfg(test)]
mod tests {
    use crate::{
        font::cosmic_font::CosmicFont,
        span::Span,
        table::{
            column::Column, maybe_span_column::MaybeSpanColumn, span_column::SpanColumn,
            width::Width,
        },
    };

    use super::ParaColumn;

    #[test]
    fn test_para_column() {
        let span = Span::from_md("There are so many words in this sentence!").unwrap();
        let mut span_column = SpanColumn::new(span, CosmicFont::default_left(), "\\font");
        span_column.start = 6;
        let full = MaybeSpanColumn::Span(&mut span_column);
        let column = Column::new(full, Width::Half);
        let para_column = ParaColumn::new(&column, None);
        assert!(matches!(para_column, ParaColumn::Text(_)));
        let para_column = ParaColumn::new(&column, Some(5));
        assert!(matches!(para_column, ParaColumn::None));
    }
}
