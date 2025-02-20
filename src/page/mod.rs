use std::path::Path;

use crate::font::tex_fonts::TexFonts;

use length::Length;
use margins::Margins;
use paper_size::PaperSize;
use serde::{Deserialize, Serialize};
use tables::Tables;

mod length;
mod margins;
mod paper_size;
mod tables;
mod unit;

/// Page layout parameters.
#[derive(Deserialize, Serialize)]
pub struct Page {
    pub paper_size: PaperSize,
    pub margins: Margins,
    pub tables: Tables,
    #[serde(skip, default = "get_default_table_width")]
    pub table_width: f32,
    #[serde(skip, default = "get_default_preamble")]
    pub preamble: String,
}

impl Page {
    pub const END_DOCUMENT: &str = "\n\\end{sloppypar}\\end{document}";

    fn get_preamble<P: AsRef<Path>>(
        fonts: &TexFonts<P>,
        paper_size: &PaperSize,
        margins: &Margins,
        tables: &Tables,
    ) -> String {
        let mut preamble = format!("\\documentclass[11pt, {}, openany]{{srcbook}}", paper_size);
        preamble += &format!(
            "\n\\usepackage[{}, {}]{{geometry}}\n\n",
            paper_size, margins
        );
        preamble += &["marginnote", "sectsty", "ragged2e", "paracol", "fontspec"]
            .iter()
            .map(|p| crate::tex!("usepackage", p))
            .collect::<Vec<String>>()
            .join("\n");

        preamble += "\n\n\\allsectionsfont{\\centering}\n";

        for (keyword, length) in ["\\parindent", "\\columnsep", "\\parfillskip", "\\tabcolsep"]
            .iter()
            .zip([
                &tables.paragraph_indent,
                &tables.column_separation,
                &tables.paragraph_fill_skip,
                &tables.tabular_column_separation,
            ])
        {
            preamble += &Self::set_length(keyword, length)
        }
        preamble += "\n\\newcommand{\\daftitle}[1]{\\centerfont{\\huge{#1}}}\n";
        for font in [&fonts.left, &fonts.center, &fonts.right].iter() {
            preamble.push_str(&font.font_family);
            preamble.push('\n');
        }
        preamble + "\n\n\\raggedbottom\n\n\\begin{document}\\begin{sloppypar}\n\n"
    }

    fn set_length(keyword: &str, length: &Length) -> String {
        format!("\n{}", crate::tex!("setlength", keyword, length))
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Page {
    fn default() -> Self {
        let margins = Margins::default();
        let table_width = margins.get_table_width();
        let tables = Tables::default();
        let paper_size = PaperSize::default();
        let preamble = Page::get_preamble(
            &TexFonts::default().unwrap(),
            &paper_size,
            &margins,
            &tables,
        );
        Self {
            paper_size,
            margins,
            tables,
            table_width,
            preamble,
        }
    }
}

fn get_default_preamble() -> String {
    Page::get_preamble(
        &TexFonts::default().unwrap(),
        &PaperSize::default(),
        &Margins::default(),
        &Tables::default(),
    )
}

fn get_default_table_width() -> f32 {
    Margins::default().get_table_width()
}
