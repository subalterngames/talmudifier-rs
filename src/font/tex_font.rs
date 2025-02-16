use std::{fs::write, io, path::Path};

#[cfg(feature = "default-fonts")]
use tempdir::TempDir;

#[cfg(feature = "default-fonts")]
use super::default_fonts::*;
use super::font_paths::FontPaths;

pub struct TexFont<P: AsRef<Path>> {
    /// The font family declaration.
    pub font_family: String,
    /// The command used to set the text to the target font, style, and size.
    pub command: String,
    pub size: f32,
    pub skip: f32,
    _directory: P,
}

impl<P: AsRef<Path>> TexFont<P> {
    pub fn new(
        name: &str,
        path: P,
        regular: &str,
        italic: Option<&str>,
        bold: Option<&str>,
        bold_italic: Option<&str>,
        size: f32,
        skip: f32,
    ) -> Self {
        const STYLES: [&str; 3] = ["ItalicFont", "BoldFont", "BoldItalicFont"];

        // The font family declaration.
        let mut font_family = format!(
            "\\newfontfamily\\{}font[Path={}, Ligatures=TeX",
            &name,
            &path.as_ref().to_str().unwrap().replace("\\", "/")
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
        let command = format!("{}\\{}", crate::tex!("fontsize", size, skip), name);
        Self {
            command,
            font_family,
            size,
            skip,
            _directory: path,
        }
    }
}

#[cfg(feature = "default-fonts")]
impl TexFont<TempDir> {
    const REGULAR: &str = "regular";
    const ITALIC: &str = "italic";
    const BOLD: &str = "bold";
    const BOLD_ITALIC: &str = "bold_italic";

    pub fn default_fonts() -> Result<(Self, Self, Self), io::Error> {
        let left = Self::default_left()?;
        let center = Self::default_center()?;
        let right = Self::default_right()?;
        Ok((left, center, right))
    }

    fn default_left() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_left")?;
        Self::dump_font(IM_FELL_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(IM_FELL_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(IM_FELL_BOLD, Self::BOLD, &directory)?;

        Ok(Self::new(
            "leftfont",
            directory,
            Self::REGULAR,
            Some(Self::ITALIC),
            Some(Self::BOLD),
            None,
            DEFAULT_SIZE,
            DEFAULT_SKIP,
        ))
    }

    fn default_center() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_center")?;
        Self::dump_font(EB_GARAMOND_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(EB_GARAMOND_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(EB_GARAMOND_BOLD, Self::BOLD, &directory)?;
        Self::dump_font(EB_GARAMOND_BOLD_ITALIC, Self::BOLD_ITALIC, &directory)?;

        Ok(Self::new(
            "leftfont",
            directory,
            Self::REGULAR,
            Some(Self::ITALIC),
            Some(Self::BOLD),
            Some(Self::BOLD_ITALIC),
            DEFAULT_SIZE,
            DEFAULT_SKIP,
        ))
    }

    fn default_right() -> Result<Self, io::Error> {
        let directory = TempDir::new("talmudifier_font_center")?;
        Self::dump_font(AVERIA_REGULAR, Self::REGULAR, &directory)?;
        Self::dump_font(AVERIA_ITALIC, Self::ITALIC, &directory)?;
        Self::dump_font(AVERIA_BOLD, Self::BOLD, &directory)?;
        Self::dump_font(AVERIA_BOLD_ITALIC, Self::BOLD_ITALIC, &directory)?;

        Ok(Self::new(
            "leftfont",
            directory,
            Self::REGULAR,
            Some(Self::ITALIC),
            Some(Self::BOLD),
            Some(Self::BOLD_ITALIC),
            DEFAULT_SIZE,
            DEFAULT_SKIP,
        ))
    }

    fn dump_font(font: &[u8], filename: &str, dir: &TempDir) -> Result<(), io::Error> {
        write(&dir.path().join(format!("{}.ttf", filename)), font)
    }
}

#[cfg(test)]
mod tests {
    use super::TexFont;

    #[test]
    fn test_default_tex_fonts() {
        TexFont::default_fonts().unwrap();
    }
}
