/// The width of a column with respect to the width of the table.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Width {
    One,
    Half,
    Third,
    TwoThirds,
}

impl Width {
    /// Returns the width of the column as a ratio of the width of the table.
    /// This is used by Cosmic Text.
    pub const fn column_ratio(&self) -> f32 {
        match self {
            Self::One => 1.,
            Self::Half => 0.5,
            Self::Third => 0.32,
            Self::TwoThirds => 0.655,
        }
    }
}
