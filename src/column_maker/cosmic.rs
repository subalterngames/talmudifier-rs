use std::{path::PathBuf, str::FromStr};

use cosmic_text::{fontdb::Source, Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

use crate::{column::width::Width, word::Word};

use super::ColumnMaker;

pub struct Cosmic<'c> {
    attrs: Attrs<'c>,
    font_system: FontSystem,
    metrics: Metrics,
    column_width: f32,
}

impl Cosmic<'_> {
    pub fn new(font_size: f32, line_height: f32, page_width: f32, width: Width) -> Self {
        let metrics = Metrics::new(font_size, line_height);
        let column_width = page_width * width.column_ratio();
        let mut font_system = FontSystem::new();

        let path = PathBuf::from_str("src/fonts/IM_Fell_French_Canon/FeFCrm2.ttf").unwrap();
        let font_id = font_system.db_mut().load_font_source(Source::File(path))[0];
        let family_name = font_system.db().face(font_id).unwrap().families[0]
            .0
            .clone();
        // Attributes indicate what font to choose
        let attrs = Attrs::new().family(Family::Name(&family_name));

        Self {
            metrics,
            column_width,
            font_system
        }
    }
}

impl ColumnMaker for Cosmic<'_> {
    fn get_num_lines(&mut self, words: &[Word]) -> usize {
        let mut buffer = Buffer::new(&mut self.font_system, self.metrics);
        // Set the width.
        buffer.set_size(&mut self.font_system, Some(self.column_width), None);
        let spans = Word::to_cosmic(words, self.attrs);

        buffer.set_rich_text(
            &mut self.font_system,
            spans.iter().map(|(s, a)| (s.as_str(), *a)),
            self.attrs,
            Shaping::Advanced,
        );
        // Create lines.
        buffer.shape_until_scroll(&mut self.font_system, true);
        // Return the number of lines.
        buffer.layout_runs().count()   
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cosmic() {

    }
}