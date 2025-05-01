#[cfg(feature = "default-fonts")]
use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{error::Error, prelude::FontMetrics};

use super::{cosmic_fonts::CosmicFonts, tex_fonts::TexFonts, Font};

#[cfg(feature = "default-fonts")]
use super::{default_tex_fonts::DefaultTexFonts, DEFAULT_ROOT_DIRECTORY};

/// Fonts for the left, center, and right columns.
#[derive(Deserialize, Serialize)]
pub struct Fonts {
    pub(super) left: Font,
    pub(super) center: Font,
    pub(super) right: Font,
    /// This is set in `default()` and it's used to determine how to create the `TexFonts` and `CosmicFonts`.
    #[cfg_attr(feature = "default-fonts", serde(skip))]
    default: bool,
}

impl Fonts {
    pub fn new(left: Font, center: Font, right: Font) -> Self {
        Self {
            left,
            center,
            right,
            default: false,
        }
    }

    fn get_cosmic_fonts_internal(&self, font_metrics: &FontMetrics) -> Result<CosmicFonts, Error> {
        Ok(CosmicFonts {
            left: self.left.to_cosmic(font_metrics)?,
            center: self.center.to_cosmic(font_metrics)?,
            right: self.right.to_cosmic(font_metrics)?,
        })
    }
}

#[cfg(feature = "default-fonts")]
impl Fonts {
    pub(crate) fn cosmic_fonts(&self, font_metrics: &FontMetrics) -> Result<CosmicFonts, Error> {
        if self.default {
            Ok(CosmicFonts::default())
        } else {
            self.get_cosmic_fonts_internal(font_metrics)
        }
    }

    /// Convert the fonts to TexFonts.
    pub(crate) fn tex_fonts(&self) -> Result<TexFonts, Error> {
        // Get default fonts.
        if self.default {
            match DefaultTexFonts::new() {
                Ok(default_tex_fonts) => Ok(default_tex_fonts.into()),
                Err(error) => Err(Error::TexFonts(error)),
            }
        } else {
            Ok(self.into())
        }
    }
}

#[cfg(not(feature = "default-fonts"))]
impl Fonts {
    pub(crate) fn cosmic_fonts(&self, font_metrics: &FontMetrics) -> Result<CosmicFonts, Error> {
        self.get_cosmic_fonts_internal(font_metrics)
    }

    pub(crate) fn tex_fonts(&self) -> Result<TexFonts, Error> {
        Ok(self.into())
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Fonts {
    fn default() -> Self {
        let directory = PathBuf::from_str(DEFAULT_ROOT_DIRECTORY).unwrap();
        Self {
            left: Font::new(&directory, "left"),
            center: Font::new(&directory, "center"),
            right: Font::new(&directory, "right"),
            default: true,
        }
    }
}
