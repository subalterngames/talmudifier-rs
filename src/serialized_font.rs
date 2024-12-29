use std::path::PathBuf;

use cosmic_text::Color;
use hex_color::HexColor;
use serde::Deserialize;
use serde_json::to_string;

use crate::font::Font;

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

impl Into<Font> for SerializedFont {
    fn into(self) -> Font {
        Font {
            regular: self.directory.join(&self.regular),
            italic: match &self.italic {
                Some(italic) => Some(self.directory.join(italic)),
                None => None,
            },
            bold: match &self.bold {
                Some(bold) => Some(self.directory.join(bold)),
                None => None,
            },
            bold_italic: match &self.bold_italic {
                Some(bold_italic) => Some(self.directory.join(bold_italic)),
                None => None,
            },
            size: self.size,
            skip: self.skip,
            cosmic_color: Color::rgba(self.color.r, self.color.g, self.color.b, self.color.a),
            latex_color: to_string(&self.color).unwrap(),
        }
    }
}
