use std::fmt::Display;

/// Raw markdown text for the left, center, and right columns.
pub struct RawText {
    pub left: String,
    pub center: String,
    pub right: String,
}

impl Display for RawText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LEFT:\n{}\n\nCENTER:\n{}\n\nRIGHT:\n{}",
            &self.left, &self.center, &self.right
        )
    }
}
