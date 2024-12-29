use std::path::PathBuf;

use include_directory::include_directory;
use serde_json::to_string;

use super::serialized_font::SerializedFont;

const DEFAULT_COLOR: &str = "#000000FF";

pub struct Font {
    pub regular: PathBuf,
    pub italic: PathBuf,
    pub bold: PathBuf,
    pub bold_italic: PathBuf,
    pub size: f32,
    pub skip: f32,
    pub color: String,
}

impl Font {
    #[cfg(feature = "default-fonts")]
    pub fn default_left() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/IM_Fell_French_Canon/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("FeFCrm2.ttf"),
            italic: directory.join("FeFCit2.ttf"),
            bold: directory.join("FeFCsc2.ttf"),
            bold_italic: directory.join("FeFCsc2.ttf"),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_center() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/EB_Garamond/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("EBGaramond-Regular.ttf"),
            italic: directory.join("EBGaramond-Italic.ttf"),
            bold: directory.join("EBGaramond-Bold.ttf"),
            bold_italic: directory.join("EBGaramond-BoldItalic.ttf"),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_right() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/Averia_Serif_Libre/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("AveriaSerifLibre-Regular.ttf"),
            italic: directory.join("AveriaSerifLibre-Italic.ttf"),
            bold: directory.join("AveriaSerifLibre-Bold.ttf"),
            bold_italic: directory.join("AveriaSerifLibre-BoldItalic.ttf"),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }
}

impl From<SerializedFont> for Font {
    fn from(value: SerializedFont) -> Self {
        let regular = value.directory.join(&value.regular);
        // Fallback: Use the regular font.
        let italic = match &value.italic {
            Some(italic) => value.directory.join(italic),
            None => regular.clone(),
        };
        // Fallback: Use the regular font.
        let bold = match &value.bold {
            Some(bold) => value.directory.join(bold),
            None => regular.clone(),
        };
        // Fallbacks: Use the bold, italic, then regular font.
        let bold_italic = match &value.bold_italic {
            Some(bold_italic) => value.directory.join(bold_italic),
            None => match &value.bold {
                Some(bold) => value.directory.join(bold),
                None => match &value.italic {
                    Some(italic) => value.directory.join(italic),
                    None => regular.clone(),
                },
            },
        };
        Font {
            regular,
            italic,
            bold,
            bold_italic,
            size: value.size,
            skip: value.skip,
            color: to_string(&value.color).unwrap(),
        }
    }
}
