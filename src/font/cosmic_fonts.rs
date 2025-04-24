use super::cosmic_font::CosmicFont;

/// Per-column Cosmic fonts.
pub struct CosmicFonts {
    pub left: CosmicFont,
    pub center: CosmicFont,
    pub right: CosmicFont,
}

#[cfg(feature = "default-fonts")]
impl Default for CosmicFonts {
    fn default() -> Self {
        Self {
            left: CosmicFont::default_left(),
            center: CosmicFont::default_center(),
            right: CosmicFont::default_right(),
        }
    }
}
