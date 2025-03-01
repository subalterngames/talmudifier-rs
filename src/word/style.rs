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
    /// e.g. if `self == Self::Bold && style == Style::Italic`, then this returns:
    /// `}\textbf{`
    pub fn get_command(&self, style: &Self) -> (Option<&'static str>, Option<&'static str>) {
        const TEXTBF: &str = "\\textbf{";
        const TEXTIT: &str = "\\textit{";

        match (self, style) {
            (Style::Regular, Style::Italic) | (Style::Bold, Style::BoldItalic) => {
                (Some(TEXTIT), None)
            }
            (Style::Regular, Style::Bold) | (Style::Italic, Style::BoldItalic) => {
                (Some(TEXTBF), None)
            }
            (Style::Regular, Style::BoldItalic) => (Some("\\textit{\\textbf{"), None),
            (Style::Italic | Style::Bold, Style::Regular) => (None, Some("}")),
            (Style::Italic, Style::Bold) => (Some(TEXTBF), Some("}")),
            (Style::Bold, Style::Italic) => (Some(TEXTIT), Some("}")),
            (Style::BoldItalic, Style::Regular) => (None, Some("}}")),
            (Style::BoldItalic, Style::Italic) => (Some(TEXTIT), Some("}}")),
            (Style::BoldItalic, Style::Bold) => (Some(TEXTBF), Some("}}")),
            (Style::Regular, Style::Regular)
            | (Style::Italic, Style::Italic)
            | (Style::Bold, Style::Bold)
            | (Style::BoldItalic, Style::BoldItalic) => (None, None),
        }
    }
}
