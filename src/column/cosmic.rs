use cosmic_text::{Buffer, FontSystem, Shaping};

use crate::{font::cosmic_font::CosmicFont, word::Word};

use super::{error::Error, width::Width, ColumnMaker};

pub struct Cosmic<'c> {
    font: &'c CosmicFont,
    column_width: f32,
    font_system: &'c mut FontSystem,
}

impl<'c> Cosmic<'c> {
    pub fn new(
        font: &'c CosmicFont,
        width: Width,
        page_width: f32,
        font_system: &'c mut FontSystem,
    ) -> Self {
        let column_width = page_width * width.column_ratio();

        Self {
            font,
            column_width,
            font_system,
        }
    }
}

impl<'c> ColumnMaker for Cosmic<'c> {
    fn get_num_lines(&mut self, words: &[Word]) -> Result<usize, Error> {
        let mut buffer = Buffer::new(&mut self.font_system, self.font.metrics);
        // Set the width.
        buffer.set_size(&mut self.font_system, Some(self.column_width), None);
        let spans = Word::to_cosmic(words, self.font);

        buffer.set_rich_text(
            &mut self.font_system,
            spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
            self.font.regular.as_attrs(),
            Shaping::Advanced,
        );
        // Create lines.
        buffer.shape_until_scroll(&mut self.font_system, true);
        // Return the number of lines.
        Ok(buffer.layout_runs().count())
    }
}

#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;

    use crate::{
        column::{width::Width, ColumnMaker},
        font::cosmic_font::CosmicFont,
        word::Word,
    };

    use super::Cosmic;

    #[test]
    fn test_cosmic() {
        let lorem = include_str!("../lorem.txt");
        let words = Word::from_md(lorem).unwrap();
        assert_eq!(words.len(), 402);
        let mut font_system = FontSystem::new();
        let font = CosmicFont::default_left(&mut font_system);
        let mut cosmic_colunmn = Cosmic::new(&font, Width::Half, 614., &mut font_system);
        let num_lines = cosmic_colunmn.get_num_lines(&words).unwrap();
        assert_eq!(num_lines, 52);
    }
}
