use super::span_column::SpanColumn;

/// A column that either has a span or is empty.
pub enum MaybeSpanColumn<'t> {
    Span(&'t mut SpanColumn),
    Empty,
}
