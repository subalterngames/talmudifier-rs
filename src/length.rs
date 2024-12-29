use std::fmt;

use crate::unit::Unit;

/// A length, in a given unit.
pub struct Length {
    pub length: f32,
    pub unit: Unit,
}

impl Length {
    /// A length in inches.
    pub fn inches(length: f32) -> Self {
        Self {
            length,
            unit: Unit::In,
        }
    }

    /// A length in ems.
    pub fn em(length: f32) -> Self {
        Self {
            length,
            unit: Unit::Em,
        }
    }

    /// A length in pts.
    pub fn pt(length: f32) -> Self {
        Self {
            length,
            unit: Unit::Pt,
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.length, self.unit)
    }
}
