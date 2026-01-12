use std::path::Path;

pub struct TexFont {
    /// The font family declaration.
    pub font_family: String,
    /// The command used to set the text to the target font, style, and size.
    pub command: String,
}

impl TexFont {
    pub fn new<P: AsRef<Path>>(
        name: &str,
        path: P,
        regular: &str,
        italic: &Option<String>,
        bold: &Option<String>,
        bold_italic: &Option<String>,
    ) -> Self {
        const STYLES: [&str; 3] = ["ItalicFont", "BoldFont", "BoldItalicFont"];

        // The font family declaration.
        let mut font_family = format!(
            "\\newfontfamily\\{}[Path={}/, Ligatures=TeX",
            &name,
            &path.as_ref().to_str().unwrap().replace("\\", "/")
        );

        // Try to add styles to the font declaration.
        let styles = [italic, bold, bold_italic]
            .iter()
            .zip(STYLES)
            .filter_map(|(f, s)| f.as_ref().map(|f| format!("{}={}", s, f)))
            .collect::<Vec<String>>()
            .join(", ");
        if !styles.is_empty() {
            font_family.push_str(", ");
            font_family.push_str(&styles);
        }

        // Add the regular style.
        font_family.push_str(&format!("]{{{}}}", regular));

        // This is the font size plus the font command.
        let command = format!("\\{}", name);
        Self {
            command,
            font_family,
        }
    }
}
