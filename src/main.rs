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

#[cfg(test)]
mod tests {
    pub(crate) fn get_test_md() -> (String, String, String) {
        let raw = include_str!("../test_text/test.md")
            .split("\n\n")
            .collect::<Vec<&str>>();
        assert_eq!(raw.len(), 3);
        (raw[0].to_string(), raw[1].to_string(), raw[2].to_string())
    }
}
