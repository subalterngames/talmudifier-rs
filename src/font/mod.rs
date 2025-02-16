pub mod cosmic_font;
#[cfg(feature = "default-fonts")]
mod default_fonts;
mod font_paths;
mod tex_font;


#[cfg(feature = "default-fonts")]
const DEFAULT_SIZE: f32 = 11.;
#[cfg(feature = "default-fonts")]
const DEFAULT_SKIP: f32 = 13.;