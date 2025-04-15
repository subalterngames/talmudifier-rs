use serde::{Deserialize, Serialize};

use crate::prelude::FontMetrics;

use super::font::Font;

/// Fonts for the left, center, and right columns.
#[derive(Deserialize, Serialize)]
pub struct Fonts {
    pub left: Font,
    pub center: Font,
    pub right: Font,
    /// Metrics shared by each font.
    pub metrics: FontMetrics,
}
