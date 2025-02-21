use serde::Deserialize;

use super::font::Font;

#[derive(Deserialize)]
pub struct Fonts {
    pub left: Font,
    pub center: Font,
    pub right: Font,
}
