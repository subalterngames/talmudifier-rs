use std::{fs::read, path::Path};

use fonts::Fonts;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use text_paths::TextPaths;

use crate::{
    error::Error,
    font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
    page::Page,
};

mod font;
mod fonts;
mod raw_text;
mod text_paths;

type CosmicFonts = Result<(CosmicFont, CosmicFont, CosmicFont), Error>;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub page: Page,
    #[cfg(feature = "default-fonts")]
    pub fonts: Option<Fonts>,
    #[cfg(not(feature = "default-fonts"))]
    pub fonts: Option<Fonts>,
    pub text_paths: TextPaths,
    pub title: Option<String>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, Error> {
        match read(path) {
            Ok(text) => match from_slice(&text) {
                Ok(config) => Ok(config),
                Err(error) => Err(Error::ConfigSerde(error)),
            },
            Err(error) => Err(Error::ConfigRead(error)),
        }
    }

    fn get_cosmic_fonts_internal(fonts: &Fonts) -> CosmicFonts {
        let left = fonts.left.to_cosmic()?;
        let center = fonts.center.to_cosmic()?;
        let right = fonts.right.to_cosmic()?;
        Ok((left, center, right))
    }

    fn get_tex_fonts_internal(fonts: &Fonts) -> TexFonts {
        let left = fonts.left.to_tex("leftfont");
        let center = fonts.center.to_tex("centerfont");
        let right = fonts.right.to_tex("rightfont");
        TexFonts {
            left,
            center,
            right,
            _default_tex_fonts: None,
        }
    }
}

#[cfg(feature = "default-fonts")]
impl Config {
    pub fn get_cosmic_fonts(&self) -> CosmicFonts {
        match &self.fonts {
            Some(fonts) => Self::get_cosmic_fonts_internal(fonts),
            None => Ok((
                CosmicFont::default_left(),
                CosmicFont::default_center(),
                CosmicFont::default_right(),
            )),
        }
    }

    pub fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        match &self.fonts {
            Some(fonts) => Ok(Self::get_tex_fonts_internal(fonts)),
            None => match TexFonts::default() {
                Ok(tex_fonts) => Ok(tex_fonts),
                Err(error) => Err(Error::TexFonts(error)),
            },
        }
    }
}

#[cfg(not(feature = "default-fonts"))]
impl Config {
    pub fn get_cosmic_fonts(&self) -> CosmicFonts {
        Self::get_cosmic_fonts_internal(&self.fonts)
    }

    pub fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        Ok(Self::get_tex_fonts_internal(&self.fonts))
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Config {
    fn default() -> Self {
        Self {
            page: Page::default(),
            fonts: None,
            text_paths: TextPaths::default(),
            title: Some("The Title".to_string()),
        }
    }
}
