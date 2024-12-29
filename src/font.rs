use std::path::PathBuf;

use cosmic_text::Color;
use include_directory::{include_directory, Dir};

const DEFAULT_LATEX_COLOR: &str = "#000000FF";
const DEFAULT_COSMIC_COLOR: Color = Color(4278190080);

pub struct Font {
    pub regular: PathBuf,
    pub italic: Option<PathBuf>,
    pub bold: Option<PathBuf>,
    pub bold_italic: Option<PathBuf>,
    pub size: f32,
    pub skip: f32,
    pub(crate) cosmic_color: Color,
    pub(crate) latex_color: String,
}

impl Font {
    pub fn default_left() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/IM_Fell_French_Canon/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("FeFCrm2.ttf"),
            italic: Some(directory.join("FeFCit2.ttf")),
            bold: Some(directory.join("FeFCsc2.ttf")),
            bold_italic: None,
            size: 11.,
            skip: 13.,
            cosmic_color: DEFAULT_COSMIC_COLOR,
            latex_color: DEFAULT_LATEX_COLOR.to_string(),
        }
    }

    pub fn default_center() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/EB_Garamond/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("EBGaramond-Regular.ttf"),
            italic: Some(directory.join("EBGaramond-Italic.ttf")),
            bold: Some(directory.join("EBGaramond-Bold.ttf")),
            bold_italic: Some(directory.join("EBGaramond-BoldItalic.ttf")),
            size: 11.,
            skip: 13.,
            cosmic_color: DEFAULT_COSMIC_COLOR,
            latex_color: DEFAULT_LATEX_COLOR.to_string(),
        }
    }

    pub fn default_right() -> Self {
        let directory = include_directory!("$CARGO_MANIFEST_DIR/src/fonts/Averia_Serif_Libre/")
            .path()
            .to_path_buf();
        Self {
            regular: directory.join("AveriaSerifLibre-Regular.ttf"),
            italic: Some(directory.join("AveriaSerifLibre-Italic.ttf")),
            bold: Some(directory.join("AveriaSerifLibre-Bold.ttf")),
            bold_italic: Some(directory.join("AveriaSerifLibre-BoldItalic.ttf")),
            size: 11.,
            skip: 13.,
            cosmic_color: DEFAULT_COSMIC_COLOR,
            latex_color: DEFAULT_LATEX_COLOR.to_string(),
        }
    }
}
