use cosmic_text::Metrics;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct FontMetrics {
    pub size: f32,
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
