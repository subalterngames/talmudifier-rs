use std::{fs::read, path::Path};

use cosmic_text::FontSystem;
use fonts::Fonts;
use serde::Deserialize;
use serde_json::{from_slice, from_str};
use text_paths::TextPaths;

use crate::{
    column::Column, daf::Daf, error::Error, font::{cosmic_font::CosmicFont, cosmic_fonts::CosmicFonts, tex_fonts::TexFonts}, page::Page, word::Word
};

mod font;
mod fonts;
mod raw_text;
mod text_paths;

#[derive(Deserialize)]
pub struct Config {
    pub page: Page,
    #[cfg(feature = "default-fonts")]
    pub fonts: Option<Fonts>,
    #[cfg(not(feature = "default-fonts"))]
    pub fonts: Option<Fonts>,
    pub text_paths: TextPaths,
    pub title: Option<String>
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

    pub fn to_daf(self, font_system: &mut FontSystem) -> Result<Daf, Error> {
        // Get the raw text.
        let raw_text = self.text_paths.read()?;

        // Get the words.
        let left_words = Word::from_md(&raw_text.left)?;
        let center_words = Word::from_md(&raw_text.center)?;
        let right_words = Word::from_md(&raw_text.right)?;

        // Get the fonts.
        let mut font_system = FontSystem::new();
        let cosmic_fonts = self.get_cosmic_fonts(&mut font_system)?;
        let tex_fonts = self.get_tex_fonts()?;

        let left = Column::new(left_words, cosmic_fonts.left, &tex_fonts.left.command, font_system);
        let center = Column::new(center_words, cosmic_fonts.center, &tex_fonts.center.command, font_system);
        let right = Column::new(right_words, cosmic_fonts.right, &tex_fonts.right.command, font_system);
        Ok(Daf {
            left,
            center,
            right,
            page: self.page,
            title: self.title
        })
    }

    fn get_cosmic_fonts_internal(
        fonts: &Fonts,
        font_system: &mut FontSystem,
    ) -> Result<CosmicFonts, Error> {
        let left = fonts.left.to_cosmic(font_system)?;
        let center = fonts.center.to_cosmic(font_system)?;
        let right = fonts.right.to_cosmic(font_system)?;
        Ok(CosmicFonts {
            left,
            center,
            right,
        })
    }

    fn get_tex_fonts_internal(fonts: &Fonts) -> TexFonts {
        let left = fonts.left.to_tex("leftfont");
        let center = fonts.center.to_tex("centerfont");
        let right = fonts.right.to_tex("rightfont");
        TexFonts {
            left,
            center,
            right,
        }
    }
}

#[cfg(feature = "default-fonts")]
impl Config {
    pub fn get_cosmic_fonts(&self, font_system: &mut FontSystem) -> Result<CosmicFonts, Error> {
        match &self.fonts {
            Some(fonts) => Self::get_cosmic_fonts_internal(fonts, font_system),
            None => Ok(CosmicFonts::default(font_system)),
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
    pub fn get_cosmic_fonts(&self, font_system: &mut FontSystem) -> Result<CosmicFonts, Error> {
        Self::get_cosmic_fonts_internal(&self.fonts, font_system)
    }

    pub fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        Ok(Self::get_tex_fonts_internal(&self.fonts))
    }
}
