use super::{position::Position, style::Style};

/// A word and its style.
#[derive(Clone)]
pub struct Word {
    /// A single word.
    pub word: String,
    /// The font style.
    pub style: Style,
    /// The position on
    pub position: Position,
}
