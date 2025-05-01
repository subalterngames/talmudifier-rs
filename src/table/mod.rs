use std::ops::Range;

use column::Column;
use cosmic_text::{Buffer, Shaping};
use maybe_span_column::MaybeSpanColumn;
use para_column::ParaColumn;
use pdf_extract::extract_text_from_mem_by_pages;
use position::Position;

use crate::{error::Error, get_pdf, page::Page, tex};

mod column;
pub(crate) mod maybe_span_column;
mod para_column;
mod position;
pub(crate) mod span_column;
mod width;

pub type OptionalColumn<'t> = Option<MaybeSpanColumn<'t>>;

macro_rules! column_ratio {
    ($($value:expr),+) => {
        {
            let mut ratios = vec![];
            $(
                ratios.push($value.to_string());
            )+

            tex!("columnratio", ratios.join(","))
        }
    };
}

/// A table is comprised of 1-3 columns of raw markdown text that can be typeset into a TeX table.
pub struct Table<'t> {
    left: Column<'t>,
    center: Column<'t>,
    right: Column<'t>,
    begin_paracol: String,
    page: &'t Page,
    log: bool,
    num_columns: usize,
}

impl<'t> Table<'t> {
    pub fn new(
        left: OptionalColumn<'t>,
        center: OptionalColumn<'t>,
        right: OptionalColumn<'t>,
        page: &'t Page,
        log: bool,
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
                column_ratio!("0.655"),
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

        let begin_paracol = format!("{}\n{}\n", ratio, begin_paracol);
        Self {
            left,
            center,
            right,
            begin_paracol,
            page,
            log,
            num_columns,
        }
    }

    /// Returns the [`Position`] of the column with the least remaining number of lines.
    /// To get the number of lines, we need to render a PDF and extract the text, which is slow.
    /// However, if there is only 1 column with text, we skip the render and this returns `(position, None)`.
    pub fn get_min_num_lines(&self) -> Result<(Option<usize>, Position), Error> {
        // If there is only one column, skip the render.
        let tex = [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .zip([&self.left, &self.center, &self.right])
            .filter_map(|(p, c)| {
                c.get_span_column()
                    .map(|column| (p, column.to_tex(None, false)))
            })
            .collect::<Vec<(Position, String)>>();
        if tex.len() == 1 {
            Ok((None, tex[0].0))
        } else {
            // Convert each column into a table.
            let para_columns = [Position::Left, Position::Center, Position::Right]
                .into_iter()
                .map(|position| self.get_paracolumns_for_num_lines(position, None))
                .collect::<Vec<[ParaColumn; 3]>>();

            if para_columns.iter().all(Self::para_columns_done) {
                Err(Error::NoColumns)
            } else {
                // Convert to paracols.
                let mut paracols = vec![];
                // We KNOW that the positions vec will match the extracted pages.
                let mut positions = vec![];
                for (para_columns, position) in para_columns.into_iter().zip([
                    Position::Left,
                    Position::Center,
                    Position::Right,
                ]) {
                    // Add the paracol.
                    if let Some(paracol) = self.get_paracol(&para_columns) {
                        paracols.push(paracol);
                        positions.push(position);
                    }
                }

                // Get the preamble.
                let mut tex = self.page.preamble.clone();

                // Add the paracols, separated by new lines.
                tex.push_str(&paracols.join("\\newpage"));

                // End the document.
                tex.push_str(Page::END_DOCUMENT);

                // Render the PDF.
                let pdf = get_pdf(&tex, self.log)?;

                // Get the number of lines per page (which is the same as per column).
                let num_lines = Self::get_extracted_line_counts(&pdf)?;
                // Get the minimum number of lines
                Ok(match num_lines.into_iter().enumerate().min_by(|a, b| a.1.cmp(&b.1)) {
                    Some((index, min_lines)) => (Some(min_lines), positions[index]),
                    None => unreachable!("We already checked that at least column has words, yet we didn't get a minimum number of lines.")
                })
            }
        }
    }

    fn get_extracted_line_counts(pdf: &[u8]) -> Result<Vec<usize>, Error> {
        // Extract text per-page.
        match extract_text_from_mem_by_pages(pdf) {
            Ok(pages) => {
                // Get the number of lines per-page.
                Ok(pages
                    .into_iter()
                    .map(|page| page.split('\n').filter(|s| !s.is_empty()).count())
                    .collect::<Vec<usize>>())
            }
            Err(error) => Err(Error::Extract(error)),
        }
    }

    fn get_extracted_line_counts_in_range(
        &self,
        position: Position,
        range: Range<usize>,
    ) -> Result<Vec<usize>, Error> {
        // Get the preamble.
        let mut tex = self.page.preamble.clone();

        let paracols = range
            .map(|end| self.get_paracolumns_for_num_lines(position, Some(end)))
            .filter_map(|para_columns| self.get_paracol(&para_columns))
            .collect::<Vec<String>>();

        // Add the paracols, separated by new lines.
        tex.push_str(&paracols.join("\\newpage"));

        // End the document.
        tex.push_str(Page::END_DOCUMENT);

        // Render the PDF.
        let pdf = get_pdf(&tex, self.log)?;

        Self::get_extracted_line_counts(&pdf)
    }

    /// Given a target `num_lines`, generate a TeX string of the table.
    /// The table will include all the words starting from `self.start` in each respective column up to `end` where `end` is an index in the underlying span of words that fills a column of `num_lines`-worth of text.
    ///
    /// To fill each column up to `num_lines`, we need to iteratively generate PDFs, extract the text, and count the number of lines. This is *slow*.
    ///
    /// `position` can be set to include only the text of a specific column. This is important when trying to get the minimum number of lines.
    ///
    /// Returns None if none of the columns have text.
    pub fn get_tex_table(
        &mut self,
        position: Option<Position>,
        num_lines: usize,
    ) -> Result<Option<String>, Error> {
        let mut para_columns: [ParaColumn; 3] = Default::default();
        for (pos, para_column) in [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .zip(para_columns.iter_mut())
        {
            *para_column = self.get_para_column(pos, position, num_lines)?;
        }

        if Self::para_columns_done(&para_columns) {
            Ok(None)
        } else {
            Ok(self.get_paracol(&para_columns))
        }
    }

    /// Get a TeX table that only has one column.
    /// Unlike [`get_tex_table(position, num_lines)`], this function doesn't generate any intermediary PDFs.
    /// This function assumes that there is exactly 1 column that contains text, and then generates a table including its text.
    /// This function is therefore much faster than the iterative approach.
    ///
    /// TL;DR this is an optimization that saves one PDF iteration.
    pub fn get_tex_table_one_column(&self) -> String {
        let mut para_columns: [ParaColumn; 3] = Default::default();
        for (pos, para_column) in [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .zip(para_columns.iter_mut())
        {
            *para_column = match self.get_column(pos).get_span_column() {
                Some(column) => ParaColumn::Text(column.to_tex(None, true)),
                None => ParaColumn::None,
            };
        }
        self.get_paracol(&para_columns).unwrap()
    }

    /// Returns a table with text on the left and right, and the title in the center.
    pub fn get_title_table(&mut self, title: &str) -> Result<Option<String>, Error> {
        const NUM_LINES: usize = 4;

        let left = self.get_para_column(Position::Left, None, NUM_LINES)?;
        let right = self.get_para_column(Position::Right, None, NUM_LINES)?;

        // The center is the title.
        // \begin{center}\centerfont{\huge{Talmudifier}}\end{center}
        let title = format!(
            "{}\\centerfont{{{}}}{}",
            tex!("begin", "center"),
            tex!("huge", &title),
            tex!("end", "center")
        );
        let center = ParaColumn::Text(title);

        Ok(self.get_paracol(&[left, center, right]))
    }

    /// Returns true if none of the columns have any further text.
    pub fn done(&self) -> bool {
        [&self.left, &self.center, &self.right]
            .iter()
            .all(|c| c.done())
    }

    fn get_para_column(
        &mut self,
        position: Position,
        target_position: Option<Position>,
        num_lines: usize,
    ) -> Result<ParaColumn, Error> {
        Ok(
            if target_position.is_some() && position == target_position.unwrap() {
                // Include all words.
                match self.get_column_tex(position, None, true) {
                    Some(tex) => ParaColumn::Text(tex),
                    None => ParaColumn::None,
                }
            } else {
                match self.get_cosmic_index(position, num_lines) {
                    Some(cosmic_index) => {
                        match self.get_tex_words(position, cosmic_index, num_lines)? {
                            Some(text) => ParaColumn::Text(text),
                            None => ParaColumn::Empty,
                        }
                    }
                    None => ParaColumn::None,
                }
            },
        )
    }

    /// Convert TeX strings per column into a TeX table.
    fn get_paracol(&self, columns: &[ParaColumn; 3]) -> Option<String> {
        const SWITCH: &str = "\n\\switchcolumn\n";
        const SWITCH_2: &str = "\n\\switchcolumn[2]\n";
        // Get the number of actual columns.
        let num_some = columns
            .iter()
            .filter(|c| !matches!(c, ParaColumn::None))
            .count();
        // If there are no columns, don't try to make a paracol.
        if num_some == 0 {
            None
        } else {
            // If there is no text, don't try to make a paracol.
            if columns
                .iter()
                .filter(|c| matches!(c, ParaColumn::Text(_)))
                .count()
                == 0
            {
                None
            } else {
                // Get the switches.
                let switches = match (&columns[0], &columns[1], &columns[2]) {
                    (
                        ParaColumn::Text(_) | ParaColumn::Empty,
                        ParaColumn::Text(_),
                        ParaColumn::Text(_) | ParaColumn::Empty,
                    ) => [Some(SWITCH), Some(SWITCH), None],

                    (
                        ParaColumn::Text(_) | ParaColumn::Empty,
                        ParaColumn::Empty,
                        ParaColumn::Text(_) | ParaColumn::Empty,
                    ) => [Some(SWITCH_2), None, None],

                    (
                        ParaColumn::None,
                        ParaColumn::Text(_) | ParaColumn::Empty,
                        ParaColumn::Text(_) | ParaColumn::Empty,
                    ) => [None, Some(SWITCH), None],

                    (
                        ParaColumn::Text(_) | ParaColumn::Empty,
                        ParaColumn::Text(_) | ParaColumn::Empty,
                        ParaColumn::None,
                    )
                    | (
                        ParaColumn::Text(_),
                        ParaColumn::None,
                        ParaColumn::Empty | ParaColumn::Text(_),
                    )
                    | (ParaColumn::Empty, ParaColumn::None, ParaColumn::Text(_)) => {
                        [Some(SWITCH), None, None]
                    }

                    (ParaColumn::Text(_), ParaColumn::None, ParaColumn::None)
                    | (ParaColumn::None, ParaColumn::Text(_), ParaColumn::None)
                    | (ParaColumn::None, ParaColumn::None, ParaColumn::Text(_)) => {
                        [None, None, None]
                    }

                    (
                        ParaColumn::Empty | ParaColumn::None,
                        ParaColumn::Empty | ParaColumn::None,
                        ParaColumn::Empty | ParaColumn::None,
                    ) => unreachable!(),
                };

                let mut table = self.begin_paracol.clone();

                // Get the columns with text.
                // Add the text to the table.
                // Add switch statements between the columns (but not at the end).
                for (para_column, switch) in columns.iter().zip(switches) {
                    // Add text.
                    if let ParaColumn::Text(text) = para_column {
                        table.push_str(text);
                    }
                    // Add a switch.
                    if let Some(switch) = switch {
                        table.push_str(switch);
                    }
                }

                // End the table.
                table.push_str("\n\\end{paracol}");
                Some(table)
            }
        }
    }

    fn get_paracolumns_for_num_lines(
        &self,
        position: Position,
        end: Option<usize>,
    ) -> [ParaColumn; 3] {
        match position {
            Position::Left => [
                ParaColumn::new(&self.left, end, false),
                ParaColumn::new_empty(&self.center),
                ParaColumn::new_empty(&self.right),
            ],
            Position::Center => [
                ParaColumn::new_empty(&self.left),
                ParaColumn::new(&self.center, end, false),
                ParaColumn::new_empty(&self.right),
            ],
            Position::Right => [
                ParaColumn::new_empty(&self.left),
                ParaColumn::new_empty(&self.center),
                ParaColumn::new(&self.right, end, false),
            ],
        }
    }

    /// Generate a TeX string from a column. The string will fill `num_lines` on the page.
    ///
    /// - `position` is used to specify the column.
    /// - `cosmic_index` is the initial end index that we calculated using Cosmic Text.
    ///
    /// This is, by far, the slowest function in Talmudifier.
    /// It needs to iterate through each remaining word in the column until `num_lines` are filled.
    /// So, it needs to perform multiple PDF renders and extracts.
    fn get_tex_words(
        &mut self,
        position: Position,
        cosmic_index: usize,
        num_lines: usize,
    ) -> Result<Option<String>, Error> {
        // This is a magic number I derived via benchmarking other values.
        const INCREMENT: usize = 20;

        // Get the target column.
        let column = self.get_column(position);

        // If we know that the column is already done, stop here.
        match column {
            Column::None => Ok(None),
            Column::Column { column, width: _ } => match column {
                MaybeSpanColumn::Empty => Ok(None),
                MaybeSpanColumn::Span(span_column) => {
                    let len = span_column.span.0.len();

                    // Add.
                    let mut done_add = false;
                    let mut end = cosmic_index;
                    let mut got_max = false;
                    while !done_add {
                        // Clamp the maximum increment.
                        let mut max = end + INCREMENT;
                        // When over the true maximum. This ends here.
                        if max > len {
                            max = len;
                            done_add = true;
                        }
                        let extracted_line_counts =
                            self.get_extracted_line_counts_in_range(position, end..max)?;

                        // We exceeeded the threshold so stop iterating here.
                        if extracted_line_counts.iter().any(|n| *n > num_lines) {
                            done_add = true;
                        }
                        // Get the maximum number of lines and check if any are the expected number of lines.
                        match extracted_line_counts
                            .into_iter()
                            .enumerate()
                            .filter(|(_, n)| *n == num_lines)
                            .map(|(i, _)| i)
                            .max()
                        {
                            Some(index) => {
                                got_max = true;
                                end += index;
                            }
                            // Continue the iteration.
                            None => {
                                if !done_add {
                                    end = max;
                                }
                            }
                        }
                    }

                    // No need to subtract if we've got a max because we're never going to get more words than we already have.
                    if !got_max {
                        // First, try subtracting words.
                        let mut done_subtract = false;
                        let mut end_subtract = cosmic_index;
                        let mut got_min = false;
                        while !done_subtract {
                            // Try to set the min at the increment.
                            let min = match end_subtract.checked_sub(INCREMENT) {
                                Some(min) => {
                                    if min > span_column.start + 1 {
                                        min
                                    }
                                    // We went below the start index. This ends here.
                                    else {
                                        done_subtract = true;
                                        span_column.start + 1
                                    }
                                }
                                None => {
                                    done_subtract = true;
                                    span_column.start + 1
                                }
                            };

                            let extracted_line_counts = self
                                .get_extracted_line_counts_in_range(position, min..end_subtract)?;

                            // We're below the threshold.
                            if extracted_line_counts.iter().any(|n| *n < num_lines) {
                                done_subtract = true;
                            }
                            // Get the minimum number of lines and check if any are the expected number of lines.
                            match extracted_line_counts
                                .into_iter()
                                .enumerate()
                                .filter(|(_, n)| *n == num_lines)
                                .map(|(i, _)| i)
                                .max()
                            {
                                Some(index) => {
                                    got_min = true;
                                    end_subtract -= index;
                                }
                                // Continue the iteration.
                                None => {
                                    if !done_subtract {
                                        end_subtract = min;
                                    }
                                }
                            }
                        }
                        // Set the end index.
                        end = if got_min {
                            end_subtract
                        }
                        // Just use all the words.
                        else {
                            len
                        };
                    }
                    Ok(self.get_column_tex(position, Some(end), true))
                }
            },
        }
    }

    /// Convert a column at `position` to a TeX including words from the column's start index to an `end` index.
    /// If `end` is None, it's set to the number of words in the column.
    /// Returns None if there aren't any remaining words in the column.
    fn get_column_tex(
        &mut self,
        position: Position,
        end: Option<usize>,
        marginalia: bool,
    ) -> Option<String> {
        let column = self.get_mut_column(position);
        match column {
            Column::Column { column, width: _ } => {
                match column {
                    MaybeSpanColumn::Span(column) => {
                        let len = column.span.0.len();
                        let end = match end {
                            Some(end) => end.min(len),
                            None => len,
                        };
                        if column.start >= end {
                            None
                        } else {
                            let text = column.to_tex(Some(end), marginalia);
                            // Increase the next start index.
                            column.start = end;
                            Some(text)
                        }
                    }
                    MaybeSpanColumn::Empty => Some(String::default()),
                }
            }
            Column::None => None,
        }
    }

    fn get_mut_column(&mut self, position: Position) -> &mut Column<'t> {
        match position {
            Position::Left => &mut self.left,
            Position::Center => &mut self.center,
            Position::Right => &mut self.right,
        }
    }

    fn get_column(&self, position: Position) -> &Column<'t> {
        match position {
            Position::Left => &self.left,
            Position::Center => &self.center,
            Position::Right => &self.right,
        }
    }

    /// Use Cosmic Text to guess the initial end index that will be used to fill a TeX column.
    fn get_cosmic_index(&mut self, position: Position, num_lines: usize) -> Option<usize> {
        let page_width = self.page.table_width;
        let separation = (self.num_columns - 1) as f32 * self.page.column_separation.get_pts();

        match self.get_mut_column(position) {
            Column::Column { column, width } => {
                match column {
                    MaybeSpanColumn::Span(column) => {
                        let len = column.span.0.len();
                        if column.start >= len {
                            None
                        } else {
                            // Iterate through the slice.
                            for end in column.start..len {
                                // Ignore marginalia.
                                if !column.is_word_in_body(end) {
                                    continue;
                                }
                                // Get the width of the column in pts.
                                let column_width = page_width * width.column_ratio() - separation;
                                // Prepare the Cosmic buffer.
                                let mut buffer = Buffer::new(
                                    &mut column.cosmic_font.font_system,
                                    column.cosmic_font.metrics,
                                );

                                // Set the width.
                                buffer.set_size(
                                    &mut column.cosmic_font.font_system,
                                    Some(column_width),
                                    None,
                                );

                                // Get the Cosmic spans.
                                let spans = column.to_cosmic(end);
                                // Set the text.
                                buffer.set_rich_text(
                                    &mut column.cosmic_font.font_system,
                                    spans.iter().map(|(s, a)| (s.as_str(), a.as_attrs())),
                                    column.cosmic_font.regular.as_attrs(),
                                    Shaping::Advanced,
                                );
                                // Create lines.
                                buffer
                                    .shape_until_scroll(&mut column.cosmic_font.font_system, true);
                                // Return the number of lines.
                                let num = buffer.layout_runs().count();
                                if num > num_lines {
                                    return if end == 0 { None } else { Some(end - 1) };
                                }
                            }
                            Some(len)
                        }
                    }
                    MaybeSpanColumn::Empty => Some(0),
                }
            }
            Column::None => None,
        }
    }

    fn para_columns_done(para_columns: &[ParaColumn; 3]) -> bool {
        para_columns.iter().all(|p| matches!(p, ParaColumn::None))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        font::{cosmic_font::CosmicFont, tex_fonts::TexFonts},
        page::Page,
        span::Span,
        table::{
            maybe_span_column::MaybeSpanColumn, position::Position, span_column::SpanColumn, Table,
        },
        tests::get_test_md,
    };

    #[test]
    fn test_cosmic_index() {
        let lorem = include_str!("../../test_text/lorem.txt");
        let span = Span::from_md(lorem).unwrap();
        assert_eq!(span.0.len(), 402);
        let cosmic_font = CosmicFont::default_left();
        let tex_fonts = TexFonts::new().unwrap();
        let mut column = SpanColumn::new(span, cosmic_font, &tex_fonts.left.command);
        let page = Page::default();
        let mut table = Table::new(
            Some(MaybeSpanColumn::Span(&mut column)),
            None,
            Some(MaybeSpanColumn::Empty),
            &page,
            false,
        );

        let cosmic_index = table.get_cosmic_index(Position::Left, 4).unwrap();
        assert_eq!(cosmic_index, 19);

        table = Table::new(
            Some(MaybeSpanColumn::Span(&mut column)),
            None,
            None,
            &page,
            false,
        );
        let cosmic_index = table.get_cosmic_index(Position::Left, 4).unwrap();
        assert_eq!(cosmic_index, 46);
    }

    #[test]
    fn test_min_num_lines() {
        let (left, center, right) = get_test_md();
        let left = Span::from_md(&left).unwrap();
        let center = Span::from_md(&center).unwrap();
        let right = Span::from_md(&right).unwrap();

        let tex_fonts = TexFonts::new().unwrap();

        let mut left_span =
            SpanColumn::new(left, CosmicFont::default_left(), &tex_fonts.left.command);
        let mut center_span = SpanColumn::new(
            center,
            CosmicFont::default_center(),
            &tex_fonts.center.command,
        );
        let mut right_span =
            SpanColumn::new(right, CosmicFont::default_right(), &tex_fonts.right.command);

        let page = Page::default();
        let table = Table::new(
            Some(MaybeSpanColumn::Span(&mut left_span)),
            Some(MaybeSpanColumn::Span(&mut center_span)),
            Some(MaybeSpanColumn::Span(&mut right_span)),
            &page,
            false,
        );

        let min_num_lines = table.get_min_num_lines().unwrap();
        assert_eq!(min_num_lines.0.unwrap(), 11);
    }
}
