use maybe_span_column::MaybeSpanColumn;

mod column;
mod maybe_span_column;
mod para_column;
mod span_column;
mod table;

type OptionalColumn<'t> = Option<MaybeSpanColumn<'t>>;
