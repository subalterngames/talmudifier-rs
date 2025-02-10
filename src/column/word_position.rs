use super::latex_command::LatexCommand;

/// The position of a word on a page.
#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub enum WordPosition {
    /// Just some regular ole text.
    #[default]
    Body,
    /// Left/right marginalia.
    Margin,
    /// The title on the page.
    Title,
}

impl WordPosition {
    /// Get a command to start or end a margin note.
    pub fn get_command(&self, position: &WordPosition) -> LatexCommand {
        match (self, position) {
            (WordPosition::Body, WordPosition::Margin) => LatexCommand {
                prefix: Some("\\\\marginnote{\\\\noindent\\\\justifying\\\\tiny "),
                suffix: None,
            },
            (WordPosition::Margin, WordPosition::Body) => LatexCommand {
                prefix: None,
                suffix: Some("}"),
            },
            _ => unreachable!(),
        }
    }
}
