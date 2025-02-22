use serde::Deserialize;

use super::length::Length;

#[derive(Clone, Deserialize)]
pub struct Tables {
    pub paragraph_indent: Length,
    pub column_separation: Length,
    pub paragraph_fill_skip: Length,
    pub tabular_column_separation: Length,
}

impl Default for Tables {
    fn default() -> Self {
        Self {
            paragraph_indent: Length::pt(0.),
            column_separation: Length::em(1.25),
            paragraph_fill_skip: Length::pt(0.),
            tabular_column_separation: Length::em(1.),
        }
    }
}
