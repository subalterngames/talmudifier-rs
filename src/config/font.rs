use std::path::{Path, PathBuf};

use cosmic_text::FontSystem;
use serde::Deserialize;

use crate::{
    error::Error,
    font::{
        cosmic_font::CosmicFont, font_metrics::FontMetrics, font_paths::FontPaths,
        tex_font::TexFont,
    },
};

#[derive(Deserialize)]
pub struct Font {
    pub directory: PathBuf,
    pub regular: String,
    pub italic: Option<String>,
    pub bold: Option<String>,
    pub bold_italic: Option<String>,
    pub metrics: FontMetrics,
}

impl Font {
    pub fn to_cosmic(&self) -> Result<CosmicFont, Error> {
        let font_paths = self.font_paths()?;
        match CosmicFont::new(&font_paths, &self.metrics, FontSystem::new()) {
            Ok(c) => Ok(c),
            Err(error) => Err(Error::CosmicFont(error)),
        }
    }

    pub fn to_tex(&self, name: &str) -> TexFont {
        TexFont::new(
            name,
            &self.directory,
            &self.regular,
            &self.italic,
            &self.bold,
            &self.bold_italic,
            &self.metrics,
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
