use std::{path::PathBuf, str::FromStr};

use column::{columns::Columns, position::Position, Column};
use cosmic_text::{
    fontdb::Source, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style, Weight,
};
use index::Index;
use markdown::{tokenize, Block, Span};
use page::{paper_size::WIDTH_PTS, Page};
use tex_span::TexSpan;

mod column;
mod index;
mod page;
pub mod prelude;
mod tex_span;
mod typesetter;

fn main() {
    let mut font_system = FontSystem::new();
    let metrics = Metrics::new(14.0, 20.0);

    // Load the font.
    let path = PathBuf::from_str("src/fonts/IM_Fell_French_Canon/FeFCrm2.ttf").unwrap();
    let font_id = font_system.db_mut().load_font_source(Source::File(path))[0];
    let family_name = font_system.db().face(font_id).unwrap().families[0]
        .0
        .clone();
    // Attributes indicate what font to choose
    let attrs = Attrs::new().family(Family::Name(&family_name));

    let column = Column {
        position: Position::Right,
        text: include_str!("test.md").to_string(),
        attrs,
    };

    let page = Page::default();

    let _ = get_cosmic_index(
        4,
        &column,
        &Columns::LeftRight,
        &page,
        &mut font_system,
        metrics,
    );

    // A Buffer provides shaping and layout for a UTF-8 string, create one per text widget
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.), None);

    let path = PathBuf::from_str("src/fonts/IM_Fell_French_Canon/FeFCit2.ttf").unwrap();
    assert!(path.exists(), "{:?}", path);
}

/// Convert a raw markdown string to TeX text spans.
fn markdown_to_tex(text: &str) -> Vec<TexSpan> {
    markdown_to_cosmic(text)
        .iter()
        .map(|value| value.into())
        .collect()
}

#[macro_export]
macro_rules! tex {
    ($command:expr, $($value:expr),+) => {
        {
            let mut t = format!("\\{}", &$command);
            $(
                t.push_str(&format!("{{{}}}", &$value));
            )+
            t
        }
    };
}
