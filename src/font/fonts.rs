use serde::{Deserialize, Serialize};

use super::Font;

/// Fonts for the left, center, and right columns.
#[derive(Deserialize, Serialize)]
pub struct Fonts {
    pub left: Font,
    pub center: Font,
    pub right: Font,
}
