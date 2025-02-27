use std::{fs::write, io, path::Path};

#[cfg(feature = "default-fonts")]
use tempdir::TempDir;

#[cfg(feature = "default-fonts")]
use super::default_fonts::*;
use super::font_metrics::FontMetrics;

pub struct TexFont {
    /// The font family declaration.
    pub font_family: String,
    /// The command used to set the text to the target font, style, and size.
    pub command: String,
    /// Sometimes we need to hold this until the `TexFont` drops.
    temp_directory: Option<TempDir>,
}

impl TexFont {
    pub fn new<P: AsRef<Path>>(
        name: &str,
        path: P,
        regular: &str,
        italic: &Option<String>,
        bold: &Option<String>,
        bold_italic: &Option<String>,
        metrics: &FontMetrics,
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
            .filter_map(|(f, s)| f.as_ref().map(|f| format!("{}={}.ttf", s, f)))
            .collect::<Vec<String>>()
            .join(", ");
        if !styles.is_empty() {
            font_family.push_str(", ");
            font_family.push_str(&styles);
        }

        // Add the regular style.
        font_family.push_str(&format!("]{{{}.ttf}}", regular));

        // This is the font size plus the font command.
        let command = format!(
            "{}\\{}",
            crate::tex!("fontsize", metrics.size, metrics.skip),
            name
        );
        Self {
            command,
            font_family,
            temp_directory: None,
        }
    }
}

#[cfg(feature = "default-fonts")]
impl TexFont {
    const REGULAR: &str = "regular";
    const ITALIC: &str = "italic";
    const BOLD: &str = "bold";
    const BOLD_ITALIC: &str = "bold_italic";

    pub fn default_left() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_left")?;
        Self::dump_font(IM_FELL_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(IM_FELL_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(IM_FELL_BOLD, Self::BOLD, &directory)?;

        let mut tex_font = Self::new(
            "leftfont",
            &directory,
            Self::REGULAR,
            &Some(Self::ITALIC.to_string()),
            &Some(Self::BOLD.to_string()),
            &None,
            &FontMetrics::default(),
        );
        tex_font.temp_directory = Some(directory);

        Ok(tex_font)
    }

    pub fn default_center() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_center")?;
        Self::dump_font(EB_GARAMOND_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(EB_GARAMOND_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(EB_GARAMOND_BOLD, Self::BOLD, &directory)?;
        Self::dump_font(EB_GARAMOND_BOLD_ITALIC, Self::BOLD_ITALIC, &directory)?;

        let mut tex_font = Self::new(
            "centerfont",
            &directory,
            Self::REGULAR,
            &Some(Self::ITALIC.to_string()),
            &Some(Self::BOLD.to_string()),
            &Some(Self::BOLD_ITALIC.to_string()),
            &FontMetrics::default(),
        );
        tex_font.temp_directory = Some(directory);

        Ok(tex_font)
    }

    pub fn default_right() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_center")?;
        Self::dump_font(AVERIA_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(AVERIA_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(AVERIA_BOLD, Self::BOLD, &directory)?;
        Self::dump_font(AVERIA_BOLD_ITALIC, Self::BOLD_ITALIC, &directory)?;

        let mut tex_font = Self::new(
            "rightfont",
            &directory,
            Self::REGULAR,
            &Some(Self::ITALIC.to_string()),
            &Some(Self::BOLD.to_string()),
            &Some(Self::BOLD_ITALIC.to_string()),
            &FontMetrics::default(),
        );
        tex_font.temp_directory = Some(directory);

        Ok(tex_font)
    }

    fn dump_font(font: &[u8], filename: &str, dir: &TempDir) -> Result<(), io::Error> {
        write(dir.path().join(format!("{}.ttf", filename)), font)
    }
}

#[cfg(test)]
mod tests {
    use super::TexFont;

    #[cfg(feature = "default-fonts")]
    #[test]
    fn test_default_font() {
        let font = TexFont::default_left().unwrap();
        let temp_dir = font.temp_directory.unwrap();
        assert!(&temp_dir.path().exists());
        assert!(temp_dir.path().join("regular.ttf").exists())
    }
}