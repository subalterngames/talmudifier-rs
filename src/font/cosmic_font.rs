use std::sync::Arc;

use cosmic_text::{fontdb::Source, Attrs, AttrsOwned, Family, FontSystem};

pub struct CosmicFont {
    pub regular: AttrsOwned,
    pub italic: AttrsOwned,
    pub bold: AttrsOwned,
    pub bold_italic: AttrsOwned,
    pub size: f32,
    pub skip: f32,
}

impl CosmicFont {
    #[cfg(feature = "default-fonts")]
    const DEFAULT_SIZE: f32 = 11.;
    #[cfg(feature = "default-fonts")]
    const DEFAULT_SKIP: f32 = 13.;

    fn new_from_bytes(
        regular: Vec<u8>,
        italic: Vec<u8>,
        bold: Vec<u8>,
        bold_italic: Vec<u8>,
        size: f32,
        skip: f32,
        font_system: &mut FontSystem,
    ) -> Self {
        let regular = Self::get_font(regular, font_system);
        let italic = Self::get_font(italic, font_system);
        let bold = Self::get_font(bold, font_system);
        let bold_italic = Self::get_font(bold_italic, font_system);
        Self {
            regular,
            italic,
            bold,
            bold_italic,
            size,
            skip,
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
    pub fn default_left(font_system: &mut FontSystem) -> Self {
        Self::new_from_bytes(
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCrm2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCit2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            11.,
            13.,
            font_system,
        )
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_center(font_system: &mut FontSystem) -> Self {
        Self::new_from_bytes(
            include_bytes!("../fonts/EB_Garamond/EBGaramond-Regular.ttf").to_vec(),
            include_bytes!("../fonts/EB_Garamond/EBGaramond-Italic.ttf").to_vec(),
            include_bytes!("../fonts/EB_Garamond/EBGaramond-Bold.ttf").to_vec(),
            include_bytes!("../fonts/EB_Garamond/EBGaramond-BoldItalic.ttf").to_vec(),
            11.,
            13.,
            font_system,
        )
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_right(font_system: &mut FontSystem) -> Self {
        Self::new_from_bytes(
            include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Regular.ttf").to_vec(),
            include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Italic.ttf").to_vec(),
            include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Bold.ttf").to_vec(),
            include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-BoldItalic.ttf").to_vec(),
            11.,
            13.,
            font_system,
        )
    }
}
