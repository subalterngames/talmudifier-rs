use column::Column;
use cosmic_text::{Buffer, Shaping};
use maybe_span_column::MaybeSpanColumn;
use para_column::ParaColumn;
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
    pub fn get_min_num_lines(&self) -> Result<(Position, Option<usize>), Error> {
        // If there is only one column, skip the render.
        let tex = [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .zip([&self.left, &self.center, &self.right])
            .filter_map(|(p, c)| match c {
                Column::Column { column, width: _ } => match column {
                    MaybeSpanColumn::Span(column) => Some((p, column.to_tex(None, false))),
                    MaybeSpanColumn::Empty => Some((p, String::default())),
                },
                Column::None => None,
            })
            .collect::<Vec<(Position, String)>>();
        if tex.len() == 1 {
            Ok((tex[0].0, None))
        } else {
            // Get the number of lines per position.
            let mut num_lines = vec![];
            [Position::Left, Position::Center, Position::Right]
                .into_iter()
                .map(|position| (position, self.get_num_lines_tex(position, None)))
                .try_for_each(|(position, num)| match num {
                    Ok(num) => {
                        if let Some(num) = num {
                            num_lines.push((position, num));
                        }
                        Ok(())
                    }
                    Err(error) => Err(error),
                })?;
            match num_lines.into_iter().min_by(|a, b| a.1.cmp(&b.1)) {
                Some(min) => Ok((min.0, Some(min.1))),
                None => Err(Error::NoColumns),
            }
        }
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
            *para_column = match self.get_column(pos) {
                Column::Column { column, width: _ } => match column {
                    MaybeSpanColumn::Span(column) => ParaColumn::Text(column.to_tex(None, true)),
                    MaybeSpanColumn::Empty => ParaColumn::Empty,
                },
                Column::None => ParaColumn::None,
            };
        }
        self.get_paracol(&para_columns).unwrap()
    }

    pub fn get_title_table(&mut self, title: &str) -> Result<Option<String>, Error> {
        let left = self.get_para_column(Position::Left, None, 4)?;
        let right = self.get_para_column(Position::Right, None, 4)?;

        // The center is the title.
        let center = ParaColumn::Text(tex!("chapter", crate::tex!("daftitle", title)));

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

    /// Get the number of lines in a column.
    ///
    /// - `position` is used to specify the column.
    /// - `end` is the end index. If None, `end` is set to the total number of words in the column.
    ///
    /// This is slow because it needs to render a PDF, extract the PDF to plaintext, and count the number of lines.
    ///
    /// Returns None if no columns have remaining text.
    fn get_num_lines_tex(
        &self,
        position: Position,
        end: Option<usize>,
    ) -> Result<Option<usize>, Error> {
        let para_columns = match position {
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
        };

        if Self::para_columns_done(&para_columns) {
            Ok(None)
        } else {
            match self.get_paracol(&para_columns) {
                Some(paracol) => {
                    // Get the preamble.
                    let mut tex = self.page.preamble.clone();

                    // Add the paracol.
                    tex.push_str(&paracol);

                    // End the document.
                    tex.push_str(Page::END_DOCUMENT);

                    Ok(Some(get_pdf(&tex, self.log, true)?.1.unwrap()))
                }
                None => Ok(None),
            }
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
        // Get the target column.
        let column = self.get_column(position);

        // If we know that the column is already done, stop here.
        if column.done() {
            Ok(None)
        } else {
            // Get the total number of words.
            let len = match column {
                Column::Column { column, width: _ } => match column {
                    MaybeSpanColumn::Span(column) => column.span.0.len(),
                    // Stop here.
                    MaybeSpanColumn::Empty => return Ok(None),
                },
                Column::None => unreachable!(),
            };

            // Set the initial end estimate to the index returned by Cosmic.
            let mut end = cosmic_index;

            // Get the current number of lines.
            match self.get_num_lines_tex(position, Some(end))? {
                Some(mut current_num_lines) => {
                    if current_num_lines > num_lines {
                        // Decrement until we have enough lines.
                        while end > 0 && current_num_lines > num_lines {
                            end -= 1;
                            match self.get_num_lines_tex(position, Some(end))? {
                                Some(n) => {
                                    current_num_lines = n;
                                    if current_num_lines < num_lines {
                                        end += 1
                                    }
                                }
                                None => return Ok(None),
                            }
                        }
                    } else {
                        // Increment until we go over.
                        while end < len - 1 && current_num_lines <= num_lines {
                            end += 1;
                            match self.get_num_lines_tex(position, Some(end))? {
                                Some(n) => {
                                    current_num_lines = n;
                                    if current_num_lines > num_lines {
                                        end -= 1
                                    }
                                }
                                None => return Ok(None),
                            }
                        }
                    }

                    Ok(if end == 0 {
                        None
                    } else {
                        self.get_column_tex(position, Some(end), true)
                    })
                }
                None => Ok(None),
            }
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
        let separation =
            (self.num_columns - 1) as f32 * self.page.tables.column_separation.get_pts();

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
        let tex_fonts = TexFonts::default().unwrap();
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
    fn test_num_lines_tex() {
        let (left, _, _) = get_test_md();
        let span = Span::from_md(&left).unwrap();
        let cosmic_font = CosmicFont::default_left();
        let tex_fonts = TexFonts::default().unwrap();
        let mut column = SpanColumn::new(span, cosmic_font, &tex_fonts.left.command);
        let page = Page::default();
        let table = Table::new(
            Some(MaybeSpanColumn::Span(&mut column)),
            None,
            Some(MaybeSpanColumn::Empty),
            &page,
            false,
        );

        let num_lines = table
            .get_num_lines_tex(Position::Left, None)
            .unwrap()
            .unwrap();
        assert_eq!(num_lines, 25);
    }

    #[test]
    fn test_min_num_lines() {
        let (left, center, right) = get_test_md();
        let left = Span::from_md(&left).unwrap();
        let center = Span::from_md(&center).unwrap();
        let right = Span::from_md(&right).unwrap();

        let tex_fonts = TexFonts::default().unwrap();

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
        assert_eq!(min_num_lines.1.unwrap(), 11);
    }
}
