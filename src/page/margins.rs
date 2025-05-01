use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::length::Length;

/// Page margins.
#[derive(Clone, Deserialize, Serialize)]
pub struct Margins {
    /// The distance from the left edge of the page to the text.
    pub left: Length,
    /// The distance from the right edge of the page to the text.
    pub right: Length,
    /// The distance from the top edge of the page to the text.
    pub top: Length,
    /// The distance from the bottom edge of the page to the text.
    pub bottom: Length,
    /// The separation distance of the last line of text and the footer.
    pub foot_skip: Length,
    /// The width of the marginalia.
    pub margin_paragraph_width: Length,
    /// An additional offset from the left or right of the page, depending on where the physical binding would be.
    pub binding_offset: Length,
}

impl Margins {
    pub(crate) fn get_table_width(&self) -> f32 {
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
