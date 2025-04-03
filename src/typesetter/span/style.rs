use cosmic_text::AttrsOwned;

use crate::font::cosmic_font::CosmicFont;

use super::LatexCommand;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub enum Style {
    #[default]
    Regular,
    Italic,
    Bold,
    BoldItalic,
}

impl Style {
    /// Returns the command required to *start* to *set* the style *from* `self` to `style`
    /// e.g. if `self == Self::Bold && style == Self::Italic`, then this returns:
    /// `}\textbf{`
    pub fn get_command(&self, style: &Self) -> LatexCommand {
        const TEXTBF: &str = "\\textbf{";
        const TEXTIT: &str = "\\textit{";

        match (self, style) {
            (Self::Regular, Self::Italic) | (Self::Bold, Self::BoldItalic) => (Some(TEXTIT), None),
            (Self::Regular, Self::Bold) | (Self::Italic, Self::BoldItalic) => (Some(TEXTBF), None),
            (Self::Regular, Self::BoldItalic) => (Some("\\textit{\\textbf{"), None),
            (Self::Italic | Self::Bold, Self::Regular) => (None, Some("}")),
            (Self::Italic, Self::Bold) => (Some(TEXTBF), Some("}")),
            (Self::Bold, Self::Italic) => (Some(TEXTIT), Some("}")),
            (Self::BoldItalic, Self::Regular) => (None, Some("}}")),
            (Self::BoldItalic, Self::Italic) => (Some(TEXTIT), Some("}}")),
            (Self::BoldItalic, Self::Bold) => (Some(TEXTBF), Some("}}")),
            (Self::Regular, Self::Regular)
            | (Self::Italic, Self::Italic)
            | (Self::Bold, Self::Bold)
            | (Self::BoldItalic, Self::BoldItalic) => (None, None),
        }
    }

    pub fn attrs(&self, font: &CosmicFont) -> AttrsOwned {
        match self {
            Self::Regular => &font.regular,
            Self::Italic => &font.italic,
            Self::Bold => &font.bold,
            Self::BoldItalic => &font.bold_italic,
        }
        .clone()
    }
}
