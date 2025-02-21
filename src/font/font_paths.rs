use std::{
    fmt,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

pub struct FontPaths {
    pub regular: PathBuf,
    pub italic: PathBuf,
    pub bold: PathBuf,
    pub bold_italic: PathBuf,
}

impl FontPaths {
    pub fn new<P: AsRef<Path> + fmt::Debug + Into<PathBuf>>(
        regular: P,
        italic: Option<P>,
        bold: Option<P>,
        bold_italic: Option<P>,
    ) -> Result<Self, io::Error> {
        let regular = Self::get_font(regular)?;
        let italic = Self::get_optional_font(italic, &regular)?;
        let bold = Self::get_optional_font(bold, &regular)?;
        let bold_italic = Self::get_optional_font(bold_italic, &bold)?;
        Ok(Self {
            regular,
            italic,
            bold,
            bold_italic,
        })
    }

    fn get_font<P: AsRef<Path> + fmt::Debug + Into<PathBuf>>(
        path: P,
    ) -> Result<PathBuf, io::Error> {
        let path = Into::<PathBuf>::into(path);
        if path.exists() {
            Ok(path)
        } else {
            Err(io::Error::new(
                ErrorKind::NotFound,
                format!("Font file not found: {:?}", path),
            ))
        }
    }

    fn get_optional_font<P: AsRef<Path> + fmt::Debug + Into<PathBuf>>(
        path: Option<P>,
        fallback: &Path,
    ) -> Result<PathBuf, io::Error> {
        match path {
            Some(path) => Self::get_font(path),
            None => Ok(fallback.to_path_buf()),
        }
    }
}
