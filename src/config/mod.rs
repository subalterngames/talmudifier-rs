use std::{fs::read, path::Path};

pub use daf::Daf;
pub use font::Font;
pub use fonts::Fonts;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
pub use source_text::SourceText;

use crate::{
    error::Error,
    font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
    get_pdf,
    page::Page,
    span::Span,
    table::{maybe_span_column::MaybeSpanColumn, span_column::SpanColumn, OptionalColumn, Table},
};

mod daf;
mod font;
mod fonts;
mod raw_text;
mod source_text;

type CosmicFonts = Result<(CosmicFont, CosmicFont, CosmicFont), Error>;

/// Set config data for the page and then generate it.
#[cfg_attr(feature = "default-fonts", derive(Default))]
#[derive(Deserialize, Serialize)]
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
    /// If true, logging is enabled.
    log: bool,
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

    /// Set the page layout parameters.
    pub fn page(mut self, page: Page) -> Self {
        self.page = page;
        self
    }

    /// Set the fonts.
    #[cfg(feature = "default-fonts")]
    pub fn fonts(mut self, fonts: Fonts) -> Self {
        self.fonts = Some(fonts);
        self
    }

    /// Set the fonts.
    #[cfg(not(feature = "default-fonts"))]
    pub fn fonts(mut self, fonts: Fonts) -> Self {
        self.fonts = fonts;
        self
    }

    /// Set the source Markdown text.
    pub fn source_text(mut self, source_text: SourceText) -> Self {
        self.source_text = source_text;
        self
    }

    /// Set the title text. By default, there is no title.
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Enable logging.
    pub fn log(mut self) -> Self {
        self.log = true;
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
        let left_span = Span::from_md(&raw_text.left)?;
        let center_span = Span::from_md(&raw_text.center)?;
        let right_span = Span::from_md(&raw_text.right)?;

        // Get the cosmic fonts.
        let (left_cosmic, center_cosmic, right_cosmic) = self.get_cosmic_fonts()?;

        // Get the columns.
        let mut left = SpanColumn::new(left_span, left_cosmic, &tex_fonts.left.command);
        let mut center = SpanColumn::new(center_span, center_cosmic, &tex_fonts.center.command);
        let mut right = SpanColumn::new(right_span, right_cosmic, &tex_fonts.right.command);

        let mut tables = vec![];

        // First four lines.
        let mut table = Table::new(
            Some(MaybeSpanColumn::Span(&mut left)),
            None,
            Some(MaybeSpanColumn::Span(&mut right)),
            &self.page,
            self.log,
        );

        let mut done = false;

        match table.get_tex_table(None, 4)? {
            Some(table) => {
                tables.push(table);
            }
            None => done = true,
        }

        if !done {
            done = table.done();
        }

        // Skip.
        if !done {
            table = Table::new(
                Some(MaybeSpanColumn::Span(&mut left)),
                Some(MaybeSpanColumn::Empty),
                Some(MaybeSpanColumn::Span(&mut right)),
                &self.page,
                self.log,
            );
            match table.get_tex_table(None, 1)? {
                Some(table) => tables.push(table),
                None => done = true,
            }
        }

        if !done {
            done = table.done();
        }

        while !done {
            // Decide which columns to use.
            let left_column = Self::get_column(&mut left);
            let center_column = Self::get_column(&mut center);
            let right_column = Self::get_column(&mut right);

            // Get the columns already done.
            let was_done = [&left_column, &center_column, &right_column]
                .iter()
                .map(|c| c.is_none())
                .collect::<Vec<bool>>();

            // Create a table.
            table = Table::new(
                left_column,
                center_column,
                right_column,
                &self.page,
                self.log,
            );

            // Get the minimum number of lines.
            let (position, num_lines) = table.get_min_num_lines()?;
            // Generate the table.
            match table.get_tex_table(Some(position), num_lines)? {
                Some(table) => tables.push(table),
                None => done = true,
            }

            if !done {
                done = table.done();
            }

            // Skip.
            if !done {
                table = Table::new(
                    Self::get_skip_column(&mut left, was_done[0]),
                    Self::get_skip_column(&mut center, was_done[1]),
                    Self::get_skip_column(&mut right, was_done[2]),
                    &self.page,
                    self.log,
                );
                match table.get_tex_table(None, 1)? {
                    Some(table) => tables.push(table),
                    None => done = true,
                }
            }

            // Re-check if we're done.
            if !done {
                done = table.done();
            }
        }

        // Build the document.
        let mut tex = self.page.preamble.clone();
        // Add the title.
        if let Some(title) = &self.title {
            tex.push_str(&crate::tex!("chapter", crate::tex!("daftitle", title)));
            tex.push('\n');
        }
        // Add the tables.
        tex.push_str(&tables.join("\n"));
        // End the document.
        tex.push_str(Page::END_DOCUMENT);

        // Generate the final PDF.
        let (pdf, _) = get_pdf(&tex, self.log, false)?;
        Ok(Daf { tex, pdf })
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

    fn get_column(span_column: &mut SpanColumn) -> OptionalColumn<'_> {
        if span_column.done() {
            None
        } else {
            Some(MaybeSpanColumn::Span(span_column))
        }
    }

    fn get_skip_column(span_column: &mut SpanColumn, was_done: bool) -> OptionalColumn<'_> {
        if span_column.done() {
            if was_done {
                None
            } else {
                Some(MaybeSpanColumn::Empty)
            }
        } else {
            Some(MaybeSpanColumn::Span(span_column))
        }
    }
}

#[cfg(feature = "default-fonts")]
impl Config {
    fn get_cosmic_fonts(&self) -> CosmicFonts {
        match &self.fonts {
            Some(fonts) => fonts.into(),
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
        (&self.fonts).into()
    }

    pub fn get_tex_fonts(&self) -> Result<TexFonts, Error> {
        Ok(Self::get_tex_fonts_internal(&self.fonts))
    }
}

impl From<&Fonts> for CosmicFonts {
    fn from(value: &Fonts) -> Self {
        let left = value.left.to_cosmic(&value.metrics)?;
        let center = value.center.to_cosmic(&value.metrics)?;
        let right = value.right.to_cosmic(&value.metrics)?;
        Ok((left, center, right))
    }
}
