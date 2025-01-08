use std::{fmt, path::Path};

use crate::{column::position::Position, tex};

/// An XeLaTeX font family.
pub struct FontFamily {
    /// The font family declaration.
    pub font_family: String,
    /// The command used to set the text to the target font, style, and size.
    pub command: String,
}

impl FontFamily {
    pub fn new<P: AsRef<Path> + fmt::Display>(
        position: &Position,
        path: P,
        regular: &str,
        bold: Option<&str>,
        italic: Option<&str>,
        bold_italic: Option<&str>,
        size: f32,
        skip: f32,
    ) -> Self {
        const STYLES: [&str; 3] = ["ItalicFont", "BoldFont", "BoldItalicFont"];

        // The name of the font command, e.g. "leftfont".
        let name = format!("{}font", position);

        // The font family declaration.
        let mut font_family = format!(
            "\\newfontfamily\\{}font[Path={}, Ligatures=TeX",
            &name, &path
        );

        // Try to add styles to the font declaration.
        let styles = [bold, italic, bold_italic]
            .iter()
            .zip(STYLES)
            .filter_map(|(f, s)| match f {
                Some(f) => Some(format!("{}={}", s, f)),
                None => None,
            })
            .collect::<Vec<String>>()
            .join(", ");
        if !styles.is_empty() {
            font_family.push_str(", ");
            font_family.push_str(&styles);
        }

        // Add the regular style.
        font_family.push_str(&format!("]{{{}}}", regular));

        // This is the font size plus the font command.
        let command = format!("{}\\{}", tex!("fontsize", size, skip), name);
        Self {
            command,
            font_family,
        }
    }
}
