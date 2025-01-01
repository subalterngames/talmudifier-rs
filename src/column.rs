use cosmic_text::Attrs;
use position::Position;

pub mod column_type;
pub mod columns;
pub mod position;

/// Text and properties of a column on the page.
pub struct Column<'a> {
    pub position: Position,
    /// The column's raw markdown text.
    /// This will change throughout the typesetting process.
    /// It will only include words that haven't been typeset.
    pub text: String,
    /// Cosmic text attributes.
    pub attrs: Attrs<'a>,
}
