use super::span_column::SpanColumn;

/// A column that either has a span or is empty.
pub enum MaybeSpanColumn<'t> {
    Span(&'t mut SpanColumn),
    /// This column will be typeset but will be empty.
    /// This is used to skip lines.
    Empty,
}
