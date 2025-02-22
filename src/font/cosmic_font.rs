use std::{fs::read, io, sync::Arc};

use cosmic_text::{fontdb::Source, Attrs, AttrsOwned, Family, FontSystem, Metrics};

#[cfg(feature = "default-fonts")]
use super::default_fonts::*;
use super::font_paths::FontPaths;

pub struct CosmicFont {
    pub regular: AttrsOwned,
    pub italic: AttrsOwned,
    pub bold: AttrsOwned,
    pub bold_italic: AttrsOwned,
    pub metrics: Metrics,
    pub font_system: FontSystem,
}

impl CosmicFont {
    pub fn new(
        paths: &FontPaths,
        size: f32,
        skip: f32,
        font_system: FontSystem,
    ) -> Result<Self, io::Error> {
        let regular = read(&paths.regular)?;
        let italic = read(&paths.italic)?;
        let bold = read(&paths.bold)?;
        let bold_italic = read(&paths.bold_italic)?;
        Ok(Self::new_from_bytes(
            regular,
            italic,
            bold,
            bold_italic,
            size,
            skip,
            font_system,
        ))
    }

    fn new_from_bytes(
        regular: Vec<u8>,
        italic: Vec<u8>,
        bold: Vec<u8>,
        bold_italic: Vec<u8>,
        size: f32,
        skip: f32,
        mut font_system: FontSystem,
    ) -> Self {
        let regular = Self::get_font(regular, &mut font_system);
        let italic = Self::get_font(italic, &mut font_system);
        let bold = Self::get_font(bold, &mut font_system);
        let bold_italic = Self::get_font(bold_italic, &mut font_system);
        let metrics = Metrics::new(size, skip);
        Self {
            regular,
            italic,
            bold,
            bold_italic,
            metrics,
            font_system,
        }
    }

    fn get_font(font: Vec<u8>, font_system: &mut FontSystem) -> AttrsOwned {
        let font_id = font_system
            .db_mut()
            .load_font_source(Source::Binary(Arc::new(font)))[0];
        let family_name = &font_system.db().face(font_id).unwrap().families[0].0;
        AttrsOwned::new(Attrs::new().family(Family::Name(&family_name)).into())
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_left() -> Self {
        Self::new_from_bytes(
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCrm2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCit2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            11.,
            13.,
            FontSystem::new(),
        )
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_center() -> Self {
        Self::new_from_bytes(
            IM_FELL_REGULAR.to_vec(),
            IM_FELL_ITALIC.to_vec(),
            IM_FELL_BOLD.to_vec(),
            IM_FELL_BOLD.to_vec(),
            11.,
            13.,
            FontSystem::new(),
        )
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_right() -> Self {
        Self::new_from_bytes(
            EB_GARAMOND_REGULAR.to_vec(),
            EB_GARAMOND_ITALIC.to_vec(),
            EB_GARAMOND_BOLD.to_vec(),
            EB_GARAMOND_BOLD_ITALIC.to_vec(),
            11.,
            13.,
            FontSystem::new(),
        )
    }
}
