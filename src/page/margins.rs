use std::fmt::Display;

use serde::Deserialize;

use super::length::Length;

#[derive(Clone, Deserialize)]
pub struct Margins {
    pub left: Length,
    pub right: Length,
    pub top: Length,
    pub bottom: Length,
    pub foot_skip: Length,
    pub margin_paragraph_width: Length,
    pub binding_offset: Length,
}

impl Margins {
    pub fn get_table_width(&self) -> f32 {
        614.295 - (self.left.get_pts() + self.right.get_pts())
    }
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            left: Length::inches(1.),
            right: Length::inches(1.),
            top: Length::inches(0.5),
            bottom: Length::inches(0.5),
            foot_skip: Length::inches(0.25),
            margin_paragraph_width: Length::em(5.),
            binding_offset: Length::inches(0.21),
        }
    }
}

impl Display for Margins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "left={}, right={}, top={}, bottom={}, footskip={}, marginparwidth={}, bindingoffset={}", self.left, self.right, self.top, self.bottom, self.foot_skip, self.margin_paragraph_width, self.binding_offset)
    }
}
