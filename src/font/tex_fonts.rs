#[cfg(feature = "default-fonts")]
use super::default_tex_fonts::DefaultTexFonts;
use super::{fonts::Fonts, tex_font::TexFont};
use crate::table::position::Position;
#[cfg(feature = "default-fonts")]
use std::io;

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
        let left = value.tex_font("left", Position::Left);
        let center = value.tex_font("center", Position::Center);
        let right = value.tex_font("right", Position::Right);
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
        let mut used_languages = Vec::new();
        let left = value.left.to_tex(Position::Left, &mut used_languages);
        let center = value.center.to_tex(Position::Center, &mut used_languages);
        let right = value.right.to_tex(Position::Right, &mut used_languages);
        Self {
            left,
            center,
            right,
            #[cfg(feature = "default-fonts")]
            _default_tex_fonts: None,
        }
    }
}
