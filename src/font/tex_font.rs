use crate::font::language::Language;
use crate::table::position::Position;
use std::path::Path;

pub struct TexFont {
    /// The font family declaration.
    pub font_family: String,
    /// The command used to set the text to the target font, style, and size.
    pub command: String,
    /// The column's language.
    pub language: Language,
}

impl TexFont {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P: AsRef<Path>>(
        path: P,
        regular: &str,
        italic: &Option<String>,
        bold: &Option<String>,
        bold_italic: &Option<String>,
        position: Position,
        language: Language,
        used_languages: &mut Vec<Language>,
    ) -> Self {
        const STYLES: [&str; 3] = ["ItalicFont", "BoldFont", "BoldItalicFont"];

        // Get the font family name.
        let name = match &language {
            // e.g. "leftfont"
            Language::English => format!("{position}font"),
            other => {
                if used_languages.contains(&language) {
                    // e.g. "leftfont"
                    format!("{position}font")
                } else {
                    // e.g. "hebrewfont"
                    format!("{other}font")
                }
            }
        };

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


        println!("{font_family}");

        let command = match &language {
            // e.g. "\leftfont"
            Language::English => format!("\\{name}"),
            other => {
                if used_languages.contains(&language) {
                    // e.g. "\hebrewfont\texthebrew{\leftfont"
                    format!("\\{other}font\\text{other}{{\\{name}")
                } else {
                    // e.g. "\hebrewfont\texthebrew{"
                    format!("\\{name}\\text{other}{{")
                }
            }
        };

        // Remember a new language.
        if !used_languages.contains(&language) {
            used_languages.push(language);
        };

        Self {
            command,
            font_family,
            language,
        }
    }
}
