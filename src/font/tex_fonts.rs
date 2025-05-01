#[cfg(feature = "default-fonts")]
use std::io;

#[cfg(feature = "default-fonts")]
use super::default_tex_fonts::DefaultTexFonts;
use super::{fonts::Fonts, tex_font::TexFont};

pub struct TexFonts {
    pub left: TexFont,
    pub center: TexFont,
    pub right: TexFont,
    #[cfg(feature = "default-fonts")]
    pub(crate) _default_tex_fonts: Option<DefaultTexFonts>,
}

#[cfg(feature = "default-fonts")]
impl TexFonts {
    pub fn new() -> Result<Self, io::Error> {
        Ok(DefaultTexFonts::new()?.into())
    }
}

#[cfg(feature = "default-fonts")]
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

impl From<&Fonts> for TexFonts {
    fn from(value: &Fonts) -> Self {
        let left = value.left.to_tex("leftfont");
        let center = value.center.to_tex("centerfont");
        let right = value.right.to_tex("rightfont");
        Self {
            left,
            center,
            right,
            #[cfg(feature = "default-fonts")]
            _default_tex_fonts: None,
        }
    }
}
