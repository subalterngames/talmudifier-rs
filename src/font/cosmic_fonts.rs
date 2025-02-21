use cosmic_text::FontSystem;

use super::cosmic_font::CosmicFont;

pub struct CosmicFonts {
    pub left: CosmicFont,
    pub center: CosmicFont,
    pub right: CosmicFont,
}

#[cfg(feature = "default-fonts")]
impl CosmicFonts {
    pub fn default(font_system: &mut FontSystem) -> Self {
        Self {
            left: CosmicFont::default_left(font_system),
            center: CosmicFont::default_center(font_system),
            right: CosmicFont::default_right(font_system),
        }
    }
}
