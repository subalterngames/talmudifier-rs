use super::LatexCommand;

/// The position of a word on a page.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub enum Position {
    /// Just some regular ole text.
    #[default]
    Body,
    /// Left/right marginalia.
    Margin,
}

impl Position {
    /// Get a command to start or end a margin note.
    pub fn get_command(&self, position: &Position) -> LatexCommand {
        match (self, position) {
            // Switch to marginalia.
            (Position::Body, Position::Margin) => (
                Some("\\\\marginnote{\\\\noindent\\\\justifying\\\\tiny "),
                None,
            ),
            // Switch back to the body.
            (Position::Margin, Position::Body) => (None, Some("}")),
            _ => unreachable!(),
        }
    }
}
