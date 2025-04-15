use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use super::{Font, DEFAULT_ROOT_DIRECTORY};

/// Fonts for the left, center, and right columns.
#[derive(Deserialize, Serialize)]
pub struct Fonts {
    pub left: Font,
    pub center: Font,
    pub right: Font,
}

impl Default for Fonts {
    fn default() -> Self {
        let directory = PathBuf::from_str(DEFAULT_ROOT_DIRECTORY).unwrap();
        Self {
            left: Font::new(&directory, "left"),
            center: Font::new(&directory, "center"),
            right: Font::new(&directory, "right"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use serde_json::from_slice;

    use super::Fonts;

    #[test]
    fn test_default_fonts() {
        from_slice::<Fonts>(&read("example_fonts.json").unwrap()).unwrap();
    }
}
