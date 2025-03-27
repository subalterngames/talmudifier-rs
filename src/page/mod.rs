use crate::font::tex_fonts::TexFonts;

pub use length::Length;
pub use margins::Margins;
pub use paper_size::PaperSize;
use serde::{Deserialize, Serialize};
pub use tables::Tables;
pub use unit::Unit;

mod length;
mod margins;
mod paper_size;
mod tables;
mod unit;

/// Page layout parameters.
#[derive(Clone, Deserialize, Serialize)]
pub struct Page {
    pub paper_size: PaperSize,
    pub margins: Margins,
    pub tables: Tables,
    #[serde(skip, default = "get_default_table_width")]
    pub table_width: f32,
    #[cfg_attr(
        feature = "default-fonts",
        serde(skip, default = "get_default_preamble")
    )]
    pub preamble: String,
}

impl Page {
    pub(crate) const END_DOCUMENT: &str = "\n\\end{sloppypar}\\end{document}";

    pub(crate) fn set_preamble(&mut self, fonts: &TexFonts) {
        self.preamble = Self::get_preamble(fonts, &self.paper_size, &self.margins, &self.tables);
    }

    fn get_preamble(
        fonts: &TexFonts,
        paper_size: &PaperSize,
        margins: &Margins,
        tables: &Tables,
    ) -> String {
        let mut preamble = format!("\\documentclass[11pt, {}, openany]{{scrbook}}", paper_size);
        preamble += &format!(
            "\n\\usepackage[{}, {}]{{geometry}}\n\n",
            paper_size, margins
        );
        preamble += &["marginnote", "sectsty", "ragged2e", "paracol", "fontspec"]
            .iter()
            .map(|p| crate::tex!("usepackage", p))
            .collect::<Vec<String>>()
            .join("\n");

        preamble += "\n\n\\allsectionsfont{\\centering}\n\\setlength\\parindent{";
        preamble.push_str(&tables.paragraph_indent.to_string());
        preamble.push('}');

        for (keyword, length) in ["\\columnsep", "\\parfillskip", "\\tabcolsep"].iter().zip([
            &tables.column_separation,
            &tables.paragraph_fill_skip,
            &tables.tabular_column_separation,
        ]) {
            preamble += &Self::set_length(keyword, length)
        }
        preamble.push('\n');
        for font in [&fonts.left, &fonts.center, &fonts.right].iter() {
            preamble.push_str(&font.font_family);
            preamble.push('\n');
        }
        preamble += "\\newcommand{\\daftitle}[1]{\\centerfont{\\huge{#1}}}\n";
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

#[cfg(feature = "default-fonts")]
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
