use std::{fs::read, path::Path};

pub use daf::Daf;
pub use font::Font;
pub use fonts::Fonts;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
pub use source_text::SourceText;
use tectonic::latex_to_pdf;

use crate::{
    column::{input_column::InputColumn, tex_column::TexColumn, Column},
    error::Error,
    font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
    page::Page,
    word::Word,
};

mod daf;
mod font;
mod fonts;
mod raw_text;
mod source_text;

type CosmicFonts = Result<(CosmicFont, CosmicFont, CosmicFont), Error>;

/// Set config data for the page and then generate it.
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "default-fonts", derive(Default))]
pub struct Config {
    /// The size of the page, margins, etc.
    page: Page,
    /// The fonts per column. If None, default fonts will be used.
    #[cfg(feature = "default-fonts")]
    fonts: Option<Fonts>,
    /// The fonts per column.
    #[cfg(not(feature = "default-fonts"))]
    fonts: Fonts,
    /// Raw markdown text that will be talmudified.
    source_text: SourceText,
    /// If not None, the title will be at the top of the page.
    title: Option<String>,
}

impl Config {
    /// Load config data from a file path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match read(path) {
            Ok(text) => match from_slice(&text) {
                Ok(config) => Ok(config),
                Err(error) => Err(Error::ConfigSerde(error)),
            },
            Err(error) => Err(Error::ConfigRead(error)),
        }
    }

    pub fn page(mut self, page: Page) -> Self {
        self.page = page;
        self
    }

    #[cfg(feature = "default-fonts")]
    pub fn fonts(mut self, fonts: Fonts) -> Self {
        self.fonts = Some(fonts);
        self
    }

    #[cfg(not(feature = "default-fonts"))]
    pub fn fonts(mut self, fonts: Fonts) -> Self {
        self.fonts = fonts;
        self
    }

    pub fn source_text(mut self, source_text: SourceText) -> Self {
        self.source_text = source_text;
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Convert raw markdown text into a Talmud page.
    /// This can take a while (on the other of minutes).
    /// Be patient!
    ///
    /// Returns a `Daf` containing the TeX string and the PDF.
    pub fn talmudify(&self) -> Result<Daf, Error> {
        // Get the TeX fonts.
        let tex_fonts = self.get_tex_fonts()?;

        // Clone the page.
        let mut page = self.page.clone();

        // Set the preamble.
        page.set_preamble(&tex_fonts);

        // Get the raw text.
        let raw_text = self.source_text.get_text()?;

        // Get the words.
        let left_words = Word::from_md(&raw_text.left)?;
        let center_words = Word::from_md(&raw_text.center)?;
        let right_words = Word::from_md(&raw_text.right)?;

        // Get the cosmic fonts.
        let (left_cosmic, center_cosmic, right_cosmic) = self.get_cosmic_fonts()?;

        // Get the columns.
        let mut left = Column::new(left_words, left_cosmic, &tex_fonts.left.command);
        let mut center = Column::new(center_words, center_cosmic, &tex_fonts.center.command);
        let mut right = Column::new(right_words, right_cosmic, &tex_fonts.right.command);

        // First four lines.
        let mut tables = vec![Column::get_tex_table(
            &mut InputColumn::Text(&mut left),
            &mut InputColumn::None,
            &mut InputColumn::Text(&mut right),
            4,
            &self.page,
        )?];

        // Skip.
        tables.push(Column::get_tex_table(
            &mut InputColumn::Text(&mut left),
            &mut InputColumn::Empty,
            &mut InputColumn::Text(&mut right),
            1,
            &self.page,
        )?);

        while !left.done() && !center.done() && !right.done() {
            // Get the columns that are and are not done.
            let left_optional = Self::get_optional_column(&left);
            let center_optional = Self::get_optional_column(&center);
            let right_optional = Self::get_optional_column(&right);

            // Get the minimum number of lines.
            let num_lines = Column::get_min_num_lines(
                left_optional,
                center_optional,
                right_optional,
                &self.page,
            )?;

            // Get all available columns.
            let mut left = Self::get_input_column(&mut left);
            let mut center = Self::get_input_column(&mut center);
            let mut right = Self::get_input_column(&mut right);

            // Create the table.
            tables.push(Column::get_tex_table(
                &mut left,
                &mut center,
                &mut right,
                num_lines,
                &self.page,
            )?);

            // Skip to the next table.
            left = Self::get_input_column_skip(left);
            center = Self::get_input_column_skip(center);
            right = Self::get_input_column_skip(right);

            // Create the table.
            tables.push(Column::get_tex_table(
                &mut left,
                &mut center,
                &mut right,
                1,
                &self.page,
            )?);
        }

        // Build the document.
        let mut tex = self.page.preamble.clone();
        // Add the title.
        if let Some(title) = &self.title {
            tex.push_str(&crate::tex!("chapter", crate::tex!("daftitle", title)));
            tex.push('\n');
        }
        for table in tables.iter() {
            tex.push_str(&TexColumn::get_table(table));
        }
        tex.push_str(Page::END_DOCUMENT);

        // Generate the final PDF.
        match latex_to_pdf(&tex) {
            Ok(pdf) => Ok(Daf { tex, pdf }),
            Err(error) => Err(Error::Pdf(error)),
        }
    }

    fn get_cosmic_fonts_internal(fonts: &Fonts) -> CosmicFonts {
        let left = fonts.left.to_cosmic()?;
        let center = fonts.center.to_cosmic()?;
        let right = fonts.right.to_cosmic()?;
        Ok((left, center, right))
    }

    fn get_tex_fonts_internal(fonts: &Fonts) -> TexFonts {
        let left = fonts.left.to_tex("leftfont");
        let center = fonts.center.to_tex("centerfont");
        let right = fonts.right.to_tex("rightfont");
        TexFonts {
            left,
            center,
            right,
            #[cfg(feature = "default-fonts")]
            _default_tex_fonts: None,
        }
    }

    fn get_optional_column(column: &Column) -> Option<&Column> {
        if column.done() {
            None
        } else {
            Some(column)
        }
    }

    fn get_input_column(column: &mut Column) -> InputColumn<'_> {
        if column.done() {
            InputColumn::None
        } else {
            InputColumn::Text(column)
        }
    }

    fn get_input_column_skip(column: InputColumn<'_>) -> InputColumn<'_> {
        match column {
            InputColumn::None => InputColumn::None,
            InputColumn::Empty => InputColumn::Empty,
            InputColumn::Text(text) => {
                // Skip.
                if text.done() {
                    InputColumn::Empty
                }
                // Include.
                else {
                    InputColumn::Text(text)
                }
            }
        }
    }
}

#[cfg(feature = "default-fonts")]
impl Config {
    fn get_cosmic_fonts(&self) -> CosmicFonts {
        match &self.fonts {
            Some(fonts) => Self::get_cosmic_fonts_internal(fonts),
            None => Ok((
                CosmicFont::default_left(),
                CosmicFont::default_center(),
                CosmicFont::default_right(),
            )),
        }
    }

    fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        match &self.fonts {
            Some(fonts) => Ok(Self::get_tex_fonts_internal(fonts)),
            None => match TexFonts::default() {
                Ok(tex_fonts) => Ok(tex_fonts),
                Err(error) => Err(Error::TexFonts(error)),
            },
        }
    }
}

#[cfg(not(feature = "default-fonts"))]
impl Config {
    pub fn get_cosmic_fonts(&self) -> CosmicFonts {
        Self::get_cosmic_fonts_internal(&self.fonts)
    }

    pub fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        Ok(Self::get_tex_fonts_internal(&self.fonts))
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Config {
    fn default() -> Self {
        Self {
            page: Page::default(),
            fonts: None,
            source_text: SourceText::default(),
            title: None,
            logger: Some(Logger::default()),
        }
    }
}
