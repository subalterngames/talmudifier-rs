use std::io;

use super::{default_tex_fonts::DefaultTexFonts, tex_font::TexFont};

pub struct TexFonts {
    pub left: TexFont,
    pub center: TexFont,
    pub right: TexFont,
    pub(crate) _default_tex_fonts: Option<DefaultTexFonts>,
}

impl From<DefaultTexFonts> for TexFonts {
    fn from(value: DefaultTexFonts) -> Self {
        let left = value.tex_font("left");
        let center = value.tex_font("center");
        let right = value.tex_font("right");
        Self {
            left,
            center,
            right,
            _default_tex_fonts: Some(value),
        }
    }
}

#[cfg(feature = "default-fonts")]
impl TexFonts {
    pub fn default() -> Result<Self, io::Error> {
        Ok(DefaultTexFonts::default()?.into())
    }
}
