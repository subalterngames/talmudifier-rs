use std::io;

use super::tex_font::TexFont;

pub struct TexFonts {
    pub left: TexFont,
    pub center: TexFont,
    pub right: TexFont,
}

#[cfg(feature = "default-fonts")]
impl TexFonts {
    pub fn default() -> Result<Self, io::Error> {
        let left = TexFont::default_left()?;
        let center = TexFont::default_center()?;
        let right = TexFont::default_right()?;
        Ok(Self {
            left,
            center,
            right,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TexFonts;

    #[test]
    fn test_default_tex_fonts() {
        TexFonts::default().unwrap();
    }
}
