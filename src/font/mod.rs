pub mod cosmic_font;
pub mod cosmic_fonts;
#[cfg(feature = "default-fonts")]
mod default_fonts;
#[cfg(feature = "default-fonts")]
pub mod default_tex_fonts;
pub mod font_metrics;
pub mod font_paths;
pub mod fonts;
pub mod tex_font;
pub mod tex_fonts;

use std::path::{Path, PathBuf};

use cosmic_text::FontSystem;
use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    font::{cosmic_font::CosmicFont, font_paths::FontPaths, tex_font::TexFont},
    prelude::FontMetrics,
};

const DEFAULT_ROOT_DIRECTORY: &str = "talmudifier_fonts";

/// The paths to the font files.
#[derive(Deserialize, Serialize)]
pub struct Font {
    pub directory: PathBuf,
    pub regular: String,
    pub italic: Option<String>,
    pub bold: Option<String>,
    pub bold_italic: Option<String>,
}

impl Font {
    pub(super) fn new(fonts_directory: &Path, folder: &str) -> Self {
        Self {
            directory: fonts_directory.join(folder),
            regular: "regular.ttf".to_string(),
            italic: Some("italic.ttf".to_string()),
            bold: Some("bold.ttf".to_string()),
            bold_italic: Some("bold_italic.ttf".to_string()),
        }
    }
    /// Create a `CosmicFont` from the font files.
    pub(super) fn to_cosmic(&self, metrics: &FontMetrics) -> Result<CosmicFont, Error> {
        let font_paths = self.font_paths()?;
        match CosmicFont::new(&font_paths, metrics, FontSystem::new()) {
            Ok(c) => Ok(c),
            Err(error) => Err(Error::CosmicFont(error)),
        }
    }

    /// Create a `TexFont` from the font files.
    pub(super) fn to_tex(&self, name: &str) -> TexFont {
        TexFont::new(
            name,
            &self.directory,
            &self.regular,
            &self.italic,
            &self.bold,
            &self.bold_italic,
        )
    }

    fn font_paths(&self) -> Result<FontPaths, Error> {
        let regular = self.get_font(&self.regular)?;
        let italic = self.get_optional_font(&self.italic, &regular)?;
        let bold = self.get_optional_font(&self.bold, &italic)?;
        let bold_italic = self.get_optional_font(&self.bold_italic, &bold)?;
        Ok(FontPaths {
            regular,
            italic,
            bold,
            bold_italic,
        })
    }

    fn get_font(&self, path: &str) -> Result<PathBuf, Error> {
        let path = self.directory.join(path);
        if path.exists() {
            Ok(path)
        } else {
            Err(Error::NoFont(path))
        }
    }

    fn get_optional_font(&self, path: &Option<String>, fallback: &Path) -> Result<PathBuf, Error> {
        match path {
            Some(path) => self.get_font(path),
            None => Ok(fallback.to_path_buf()),
        }
    }
}
