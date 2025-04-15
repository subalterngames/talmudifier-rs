use cosmic_text::Metrics;
use serde::{Deserialize, Serialize};

/// Metrics for each font on the page.
#[derive(Clone, Deserialize, Serialize)]
pub struct FontMetrics {
    /// The size of the font.
    pub size: f32,
    /// The height of the line skip.
    pub skip: f32,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            size: 11.,
            skip: 13.,
        }
    }
}

impl From<&FontMetrics> for Metrics {
    fn from(value: &FontMetrics) -> Self {
        Metrics::new(value.size, value.skip)
    }
}
