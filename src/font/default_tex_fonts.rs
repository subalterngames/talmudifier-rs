use super::{default_fonts::*, tex_font::TexFont, Font, DEFAULT_ROOT_DIRECTORY};
use crate::font::language::Language;
use crate::table::position::Position;
use std::{
    fs::{create_dir_all, write},
    io,
    path::PathBuf,
    str::FromStr,
};

/// XeTeX requires fonts to be saved to a path (rather than exist in memory).
/// When this struct is created, it writes the fonts stored in the binary to disk in a folder.
/// When this struct is dropped, that folder is deleted.
pub struct DefaultTexFonts {
    directory: PathBuf,
}

impl DefaultTexFonts {
    pub fn new() -> Result<Self, io::Error> {
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
            directory: PathBuf::from_str(DEFAULT_ROOT_DIRECTORY).unwrap(),
        })
    }

    pub(crate) fn tex_font(&self, folder: &str, position: Position) -> TexFont {
        let font = Font::new(&self.directory, folder);
        font.to_tex(position, &mut Vec::new())
    }

    fn dump_fonts(
        folder: &str,
        regular: &[u8],
        italic: &[u8],
        bold: &[u8],
        bold_italic: Option<&[u8]>,
    ) -> Result<(), io::Error> {
        let directory = PathBuf::from_str(DEFAULT_ROOT_DIRECTORY)
            .unwrap()
            .join(folder);

        create_dir_all(&directory)?;
        write(directory.join("regular.ttf"), regular)?;
        write(directory.join("italic.ttf"), italic)?;
        write(directory.join("bold.ttf"), bold)?;
        let bold_italic = bold_italic.unwrap_or(bold);
        write(directory.join("bold_italic.ttf"), bold_italic)?;

        Ok(())
    }
}

// While testing, we can't delete the folder due to test concurrency.

#[cfg(not(test))]
impl Drop for DefaultTexFonts {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
