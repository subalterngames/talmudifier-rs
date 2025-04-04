use pdf_extract::extract_text_from_mem;
use tectonic::latex_to_pdf;

use crate::{error::Error, page::Page, tex};

use super::{
    column::Column, maybe_span_column::MaybeSpanColumn, para_column::ParaColumn, OptionalColumn,
};

macro_rules! column_ratio {
    ($($value:expr),+) => {
        {
            let mut t = "\\columnratio".to_string();
            $(
                t.push_str(&format!("{{{}}}", &$value));
            )+
            t
        }
    };
}

pub struct Table<'t> {
    left: Column<'t>,
    center: Column<'t>,
    right: Column<'t>,
    begin_paracol: String,
    page: &'t Page,
}

impl<'t> Table<'t> {
    pub fn new(
        left: OptionalColumn<'t>,
        center: OptionalColumn<'t>,
        right: OptionalColumn<'t>,
        page: &'t Page,
    ) -> Self {
        const THIRD: &str = "0.32";
        const HALF: &str = "0.5";

        // Get the number of span/empty columns (excluding non-columns).
        let num_columns = [&left, &center, &right]
            .iter()
            .filter(|c| c.is_some())
            .count();
        // Get the paracol begin command.
        let begin_paracol = tex!("begin", "paracol", num_columns);

        // Convert the columns into columns with widths. `ratio` will be used for TeX and in some cases it's a magic number.
        let (left, center, right, ratio) = match (left, center, right) {
            (Some(left), Some(center), Some(right)) => (
                Column::third(left),
                Column::third(center),
                Column::third(right),
                column_ratio!(THIRD, THIRD, THIRD),
            ),
            (Some(left), Some(center), None) => (
                Column::third(left),
                Column::two_thirds(center),
                Column::None,
                column_ratio!("0.31"),
            ),
            (Some(left), None, Some(right)) => (
                Column::half(left),
                Column::None,
                Column::half(right),
                column_ratio!(HALF, HALF),
            ),
            (None, Some(center), Some(right)) => (
                Column::two_thirds(center),
                Column::third(right),
                Column::None,
                column_ratio!("0.675"),
            ),
            (Some(left), None, None) => (
                Column::one(left),
                Column::None,
                Column::None,
                column_ratio!(1),
            ),
            (None, Some(center), None) => (
                Column::None,
                Column::one(center),
                Column::None,
                column_ratio!(1),
            ),
            (None, None, Some(right)) => (
                Column::None,
                Column::None,
                Column::one(right),
                column_ratio!(1),
            ),
            (None, None, None) => (Column::None, Column::None, Column::None, String::default()),
        };

        let begin_paracol = format!("{}\n{}\n", begin_paracol, ratio);
        Self {
            left,
            center,
            right,
            begin_paracol,
            page,
        }
    }

    /// Convert TeX strings per column into a TeX table.
    fn get_paracol(&self, left: ParaColumn, center: ParaColumn, right: ParaColumn) -> String {
        const SWITCH: &str = "\\switchcolumn ";
        // Get the number of actual columns.
        let num_some = [&left, &center, &right]
            .iter()
            .filter(|c| !matches!(c, ParaColumn::None))
            .count();
        // If there are no columns, don't try to make a paracol.
        if num_some == 0 {
            String::default()
        } else {
            let mut table = self.begin_paracol.clone();
            // Switch this many time.
            let mut num_switches = num_some - 1;

            // Get the columns with text.
            // Add the text to the table.
            // Add switch statements between the columns (but not at the end).
            for para_column in [&left, &center, &right].iter() {
                match para_column {
                    ParaColumn::Text(text) => {
                        // Add the text.
                        table.push_str(text);
                        // Switch columns.
                        if num_switches > 0 {
                            table.push_str(SWITCH);
                            num_switches -= 1;
                        }
                    }
                    ParaColumn::Empty => {
                        // Switch columns.
                        if num_switches > 0 {
                            table.push_str(SWITCH);
                            num_switches -= 1;
                        }
                    }
                    ParaColumn::None => ()
                }
            }
            
            // End the table.
            table.push_str("\n\n\\end{paracol}");
            table
        }
    }

    /// Get the number of lines in a column using Tectonic.
    ///
    /// - `end` is an optional end index for `self.words`. If `None`, the words are from `self.start..self.words.len()`.
    /// - `width` is the width of the column, as calculated elsewhere.
    /// - `page` is used because we need the preamble.
    fn get_num_lines_tex(
        &'t self,
        column: &'t Column<'t>,
        end: Option<usize>,
    ) -> Result<Option<usize>, Error> {
        match column {
            Column::Column { column, width: _ } => {
                match column {
                    MaybeSpanColumn::Span(column) => {
                        // Get the TeX string.
                        // Get the preamble.
                        let mut tex = self.page.preamble.clone();

                        *text = column.to_tex(end);

                        tex.push_str(&self.get_paracol());

                        *text = temp;

                        // End the document.
                        tex.push_str(Page::END_DOCUMENT);

                        // Create a PDF.
                        match latex_to_pdf(&tex) {
                            // Extract the text of the PDF.
                            Ok(pdf) => match extract_text_from_mem(&pdf) {
                                Ok(text) => Ok(Some(text.split('\n').count())),
                                Err(error) => Err(Error::Extract(error)),
                            },
                            Err(error) => {
                                // Dump the bad tex.
                                #[cfg(test)]
                                std::fs::write("test_text/bad.tex", tex).unwrap();

                                Err(Error::Pdf(error))
                            }
                        }
                    }
                    MaybeSpanColumn::Empty => Ok(None),
                }
            }
            Column::None => Ok(None),
        }
    }
}
