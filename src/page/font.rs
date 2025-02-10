use std::fs::read;

use serde_json::to_string;

use super::serialized_font::SerializedFont;

const DEFAULT_COLOR: &str = "#000000FF";

pub struct Font {
    pub regular: Vec<u8>,
    pub italic: Vec<u8>,
    pub bold: Vec<u8>,
    pub bold_italic: Vec<u8>,
    pub size: f32,
    pub skip: f32,
    pub color: String,
}

impl Font {
    #[cfg(feature = "default-fonts")]
    pub fn default_left() -> Self {
        Self {
            regular: include_bytes!("../fonts/IM_Fell_French_Canon/FeFCrm2.ttf").to_vec(),
            italic: include_bytes!("../fonts/IM_Fell_French_Canon/FeFCit2.ttf").to_vec(),
            bold: include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            bold_italic: include_bytes!("../fonts/IM_Fell_French_Canon/FeFCsc2.ttf").to_vec(),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_center() -> Self {
        Self {
            regular: include_bytes!("../fonts/EB_Garamond/EBGaramond-Regular.ttf").to_vec(),
            italic: include_bytes!("../fonts/EB_Garamond/EBGaramond-Italic.ttf").to_vec(),
            bold: include_bytes!("../fonts/EB_Garamond/EBGaramond-Bold.ttf").to_vec(),
            bold_italic: include_bytes!("../fonts/EB_Garamond/EBGaramond-BoldItalic.ttf").to_vec(),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }

    #[cfg(feature = "default-fonts")]
    pub fn default_right() -> Self {
        Self {
            regular: include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Regular.ttf")
                .to_vec(),
            italic: include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Italic.ttf")
                .to_vec(),
            bold: include_bytes!("../fonts/Averia_Serif_Libre/AveriaSerifLibre-Bold.ttf").to_vec(),
            bold_italic: include_bytes!(
                "../fonts/Averia_Serif_Libre/AveriaSerifLibre-BoldItalic.ttf"
            )
            .to_vec(),
            size: 11.,
            skip: 13.,
            color: DEFAULT_COLOR.to_string(),
        }
    }
}

impl From<SerializedFont> for Font {
    fn from(value: SerializedFont) -> Self {
        let regular = read(value.directory.join(&value.regular)).unwrap();
        // Fallback: Use the regular font.
        let italic = match &value.italic {
            Some(italic) => read(value.directory.join(italic)).unwrap(),
            None => regular.clone(),
        };
        // Fallback: Use the regular font.
        let bold = match &value.bold {
            Some(bold) => read(value.directory.join(bold)).unwrap(),
            None => regular.clone(),
        };
        // Fallbacks: Use the bold, italic, then regular font.
        let bold_italic = match &value.bold_italic {
            Some(bold_italic) => read(value.directory.join(bold_italic)).unwrap(),
            None => match &value.bold {
                Some(_) => bold.clone(),
                None => match &value.italic {
                    Some(_) => italic.clone(),
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "default-fonts")]
    fn test_default_fonts_exist() {
        use super::Font;

        let _ = Font::default_left();
        let _ = Font::default_center();
        let _ = Font::default_right();
    }
}
