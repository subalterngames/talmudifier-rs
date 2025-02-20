use std::fs::write;

use cosmic_text::FontSystem;
use error::Error;
use font::{cosmic_font::CosmicFont, tex_font::TexFont};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use word::Word;

mod column;
pub(crate) mod error;
pub(crate) mod font;
pub(crate) mod page;
pub(crate) mod word;

fn main() {
    let md = include_str!("test.md");
    let page = Page::default();
    let (left, center, right) = TexFont::default_fonts().unwrap();
    let fonts = [&left, &center, &right];
    let preamble = page.get_preamble(fonts);

    let width = Width::Half;

    let mut font_system = FontSystem::new();
    let cosmic_font = CosmicFont::default_left(&mut font_system);
    let table_width = page.get_table_width();
    let mut cosmic = Cosmic::new(&cosmic_font, width, table_width, &mut font_system);

    let words = Word::from_md(md).unwrap();

    let index = cosmic.get_words(&words, 4).unwrap().unwrap();

    /*
    let mut tex = Tex {
        preamble: &preamble,
        font: &left,
        width,
    };

    // TODO make the tex decrement.

    let index = tex.get_words(&words, 4).unwrap().unwrap();

    let (column, _) = Word::to_tex(&words[..index], &left.command);
    let tex = tex.get_tex(column);

    let pdf = latex_to_pdf(&tex).unwrap();
    write("out.pdf", pdf).unwrap();*/
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
