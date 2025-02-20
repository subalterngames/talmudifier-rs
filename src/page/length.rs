use std::fmt;

use serde::{Deserialize, Serialize};

use super::unit::Unit;

/// A length, in a given unit.
#[derive(Deserialize, Serialize)]
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

    /// Returns the length in pts.
    pub fn get_pts(&self) -> f32 {
        const PTS_IN: f32 = 72.27;
        const PTS_MM: f32 = 2.85;
        const PTS_CM: f32 = PTS_MM * 10.;

        match &self.unit {
            Unit::Pt => self.length,
            Unit::Mm => self.length * PTS_MM,
            Unit::Cm => self.length * PTS_CM,
            Unit::In => self.length * PTS_IN,
            other => panic!("Sorry, can't directly convert {} to pts", other),
        }
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.length, self.unit)
    }
}
