use crate::{font::tex_fonts::TexFonts, prelude::FontMetrics, tex};

pub use length::Length;
pub use margins::Margins;
pub use paper_size::PaperSize;
use serde::{Deserialize, Serialize};
pub use unit::Unit;

mod length;
mod margins;
mod paper_size;
mod unit;

/// Page layout parameters.
#[derive(Clone, Deserialize, Serialize)]
pub struct Page {
    /// The overall size of the page.
    pub paper_size: PaperSize,
    /// Margin distances.
    pub margins: Margins,
    /// The horizontal distance between columns.
    pub column_separation: Length,
    /// The font size and line skip.
    pub font_metrics: FontMetrics,
    /// The width of the text portion of the page.
    #[serde(skip, default = "get_default_table_width")]
    pub(crate) table_width: f32,
    /// The preamble text.
    #[cfg_attr(
        feature = "default-fonts",
        serde(skip, default = "get_default_preamble")
    )]
    pub(crate) preamble: String,
}

impl Page {
    pub(crate) const END_DOCUMENT: &str = "\n\\end{sloppypar}\\end{document}";

    pub(crate) fn set_table_width(&mut self) {
        self.table_width =
            self.paper_size.width() - (self.margins.left.get_pts() + self.margins.right.get_pts());
    }

    pub(crate) fn set_preamble(&mut self, fonts: &TexFonts) {
        self.preamble = Self::get_preamble(
            fonts,
            &self.paper_size,
            &self.margins,
            &self.column_separation,
            &self.font_metrics,
        );
    }

    fn get_preamble(
        fonts: &TexFonts,
        paper_size: &PaperSize,
        margins: &Margins,
        column_separation: &Length,
        font_metrics: &FontMetrics,
    ) -> String {
        let mut preamble = format!("\\documentclass[11pt, {}, openany]{{scrbook}}", paper_size);
        preamble += &format!(
            "\n\\usepackage[{}, {}]{{geometry}}\n\\pagenumbering{{gobble}}\n\n",
            paper_size, margins
        );
        preamble += &["marginnote", "sectsty", "ragged2e", "paracol", "fontspec"]
            .iter()
            .map(|p| crate::tex!("usepackage", p))
            .collect::<Vec<String>>()
            .join("\n");

        preamble += "\n\n\\allsectionsfont{\\centering}\n\\setlength\\parindent{";
        preamble.push_str(&Length::pt(0.).to_string());
        preamble.push('}');

        for (keyword, length) in ["\\columnsep", "\\parfillskip"]
            .iter()
            .zip([column_separation, &Length::pt(0.)])
        {
            preamble += &Self::set_length(keyword, length)
        }
        preamble.push('\n');
        for font in [&fonts.left, &fonts.center, &fonts.right].iter() {
            preamble.push_str(&font.font_family);
            preamble.push('\n');
        }
        preamble += "\n\n\\raggedbottom\n\n\\begin{document}\\begin{sloppypar}\n\n";
        preamble + &tex!("fontsize", font_metrics.size, font_metrics.skip)
    }

    fn set_length(keyword: &str, length: &Length) -> String {
        format!("\n{}", tex!("setlength", keyword, length))
    }

    #[cfg(feature = "default-fonts")]
    fn default_column_separation() -> Length {
        Length::inches(0.25)
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Page {
    fn default() -> Self {
        let margins = Margins::default();
        let paper_size = PaperSize::default();
        let font_metrics = FontMetrics::default();
        let column_separation = Self::default_column_separation();
        let table_width = get_default_table_width();

        let preamble = Page::get_preamble(
            &TexFonts::new().unwrap(),
            &paper_size,
            &margins,
            &column_separation,
            &font_metrics,
        );
        Self {
            paper_size,
            margins,
            column_separation,
            table_width,
            preamble,
            font_metrics,
        }
    }
}

#[cfg(feature = "default-fonts")]
fn get_default_preamble() -> String {
    Page::get_preamble(
        &TexFonts::new().unwrap(),
        &PaperSize::default(),
        &Margins::default(),
        &Page::default_column_separation(),
        &FontMetrics::default(),
    )
}

fn get_default_table_width() -> f32 {
    let margins = Margins::default();
    PaperSize::Letter.width() - (margins.left.get_pts() + margins.right.get_pts())
}
