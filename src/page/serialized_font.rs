use std::path::PathBuf;

use hex_color::HexColor;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SerializedFont {
    pub directory: PathBuf,
    pub regular: String,
    pub italic: Option<String>,
    pub bold: Option<String>,
    pub bold_italic: Option<String>,
    pub size: f32,
    pub skip: f32,
    pub color: HexColor,
}
