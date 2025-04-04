use crate::{font::cosmic_font::CosmicFont, span::Span};

/// A column of text that can be typeset.
/// Columns try to fill a target number of lines with words.
/// Cosmic is used to get an initial guess at the number of words.
/// Then, Tectonic is used to fill the column.
///
/// `SpanColumn` has a `start` index that are continuously re-sliced for typesetting.
pub struct SpanColumn {
    /// All of the words in the column.
    span: Span,
    /// The start index of the `words` slice.
    start: usize,
    /// The font used in Cosmic.
    cosmic_font: CosmicFont,
    /// The command to set the TeX font.
    tex_font: String,
}
