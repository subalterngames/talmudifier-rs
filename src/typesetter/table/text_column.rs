use super::span_column::SpanColumn;

/// A column that might contain text.
pub enum TextColumn {
    /// A column with text.
    Span(SpanColumn),
    /// An empty column.
    Empty,
}

impl From<SpanColumn> for TextColumn {
    fn from(value: SpanColumn) -> Self {
        Self::Span(value)
    }
}