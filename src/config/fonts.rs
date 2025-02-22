use serde::{Deserialize, Serialize};

use super::font::Font;

#[derive(Deserialize, Serialize)]
pub struct Fonts {
    pub left: Font,
    pub center: Font,
    pub right: Font,
}
