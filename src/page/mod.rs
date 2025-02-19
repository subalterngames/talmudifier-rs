use std::path::Path;

use crate::font::tex_font::TexFont;

use length::Length;
use paper_size::PaperSize;
pub use paper_size::WIDTH_PTS;

mod length;
mod paper_size;
mod unit;

/// Page layout parameters.
pub struct Page {
    pub paper_size: PaperSize,
    pub binding_offset: Length,
    pub left_margin: Length,
    pub right_margin: Length,
    pub top_margin: Length,
    pub bottom_margin: Length,
    pub foot_skip: Length,
    pub margin_paragraph_width: Length,
    pub paragraph_indent: Length,
    pub column_separation: Length,
    pub paragraph_fill_skip: Length,
    pub tabular_column_separation: Length,
}

impl Page {
    pub const END_DOCUMENT: &str = "\n\\end{sloppypar}\\end{document}";

    pub fn get_preamble<P: AsRef<Path>>(&self, fonts: [&TexFont<P>; 3]) -> String {
        let mut preamble = format!(
            "\\documentclass[11pt, {}, openany]{{srcbook}}",
            self.paper_size
        );
        preamble += &format!("\n\\usepackage[{}, bindingoffset={}, left={}, right={}, top={}, bottom={}, footskip={}, marginparwidth={}]{{geometry}}\n\n", self.paper_size, self.binding_offset, self.left_margin, self.right_margin, self.top_margin, self.bottom_margin, self.foot_skip, self.margin_paragraph_width);
        preamble += &["marginnote", "sectsty", "ragged2e", "paracol", "fontspec"]
            .iter()
            .map(|p| crate::tex!("usepackage", p))
            .collect::<Vec<String>>()
            .join("\n");

        preamble += "\n\n\\allsectionsfont{\\centering}\n";

        for (keyword, length) in ["\\parindent", "\\columnsep", "\\parfillskip", "\\tabcolsep"]
            .iter()
            .zip([
                &self.paragraph_indent,
                &self.column_separation,
                &self.paragraph_fill_skip,
                &self.tabular_column_separation,
            ])
        {
            preamble += &Self::set_length(keyword, length)
        }
        preamble += "\n\\newcommand{\\daftitle}[1]{\\centerfont{\\huge{#1}}}\n";
        for font in fonts.iter() {
            preamble.push_str(&font.command);
            preamble.push('\n');
        }
        preamble + "\n\n\\raggedbottom\n\n\\begin{document}\\begin{sloppypar}\n\n"
    }

    pub fn get_table_width(&self) -> f32 {
        WIDTH_PTS - (self.left_margin.get_pts() + self.right_margin.get_pts())
    }

    fn set_length(keyword: &str, length: &Length) -> String {
        format!("\n{}", crate::tex!("setlength", keyword, length))
    }
}

#[cfg(feature = "default-fonts")]
impl Default for Page {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::Letter,
            binding_offset: Length::inches(0.21),
            left_margin: Length::inches(1.),
            right_margin: Length::inches(1.),
            top_margin: Length::inches(0.5),
            bottom_margin: Length::inches(0.5),
            foot_skip: Length::inches(0.25),
            margin_paragraph_width: Length::em(5.),
            paragraph_indent: Length::pt(0.),
            column_separation: Length::em(1.25),
            paragraph_fill_skip: Length::pt(0.),
            tabular_column_separation: Length::em(1.),
        }
    }
}
