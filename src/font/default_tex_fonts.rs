use std::{
    fs::{create_dir_all, write},
    io,
    path::PathBuf,
    str::FromStr,
};

use super::{default_fonts::*, font_metrics::FontMetrics, tex_font::TexFont};

pub struct DefaultTexFonts {
    directory: PathBuf,
}

impl DefaultTexFonts {
    const ROOT_DIRECTORY: &str = "talmudifier_fonts";

    pub fn default() -> Result<Self, io::Error> {
        Self::dump_fonts("left", IM_FELL_REGULAR, IM_FELL_ITALIC, IM_FELL_BOLD, None)?;
        Self::dump_fonts(
            "center",
            EB_GARAMOND_REGULAR,
            EB_GARAMOND_ITALIC,
            EB_GARAMOND_BOLD,
            Some(EB_GARAMOND_BOLD_ITALIC),
        )?;
        Self::dump_fonts(
            "right",
            AVERIA_REGULAR,
            AVERIA_ITALIC,
            AVERIA_BOLD,
            Some(AVERIA_BOLD_ITALIC),
        )?;
        Ok(Self {
            directory: PathBuf::from_str(Self::ROOT_DIRECTORY).unwrap(),
        })
    }

    pub(crate) fn tex_font(&self, folder: &str) -> TexFont {
        TexFont::new(
            &format!("{}font", folder),
            self.directory.join(folder),
            "regular",
            &Some("italic".to_string()),
            &Some("bold".to_string()),
            &Some("bold_italic".to_string()),
            &FontMetrics::default(),
        )
    }

    fn dump_fonts(
        folder: &str,
        regular: &[u8],
        italic: &[u8],
        bold: &[u8],
        bold_italic: Option<&[u8]>,
    ) -> Result<(), io::Error> {
        let directory = PathBuf::from_str(Self::ROOT_DIRECTORY)
            .unwrap()
            .join(folder);

        create_dir_all(&directory)?;
        write(directory.join("regular.ttf"), regular)?;
        write(directory.join("italic.ttf"), italic)?;
        write(directory.join("bold.ttf"), bold)?;
        let bold_italic = match bold_italic {
            Some(bold_italic) => bold_italic,
            None => bold,
        };
        write(directory.join("bold_italic.ttf"), bold_italic)?;

        Ok(())
    }
}

/*
impl Drop for DefaultTexFonts {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.directory);
    }
}
    */
