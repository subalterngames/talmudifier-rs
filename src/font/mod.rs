use cosmic_font::CosmicFont;
use tex_font::TexFont;

pub mod cosmic_font;
#[cfg(feature = "default-fonts")]
mod default_fonts;
#[cfg(feature = "default-fonts")]
mod default_tex_fonts;
pub mod font_metrics;
pub mod font_paths;
pub mod tex_font;
pub mod tex_fonts;

pub struct Font {
    pub cosmic: CosmicFont,
    pub tex: TexFont,
}
