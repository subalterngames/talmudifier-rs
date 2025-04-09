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
            log,
            num_columns,
        }
    }

    /// Given 1-3 columns:
    ///
    /// 1. Get the width of each column.
    ///    This is derived from the position of each column e.g. `left` and `center` is always (one third, two thirds).
    /// 2. Get the number of lines per column, using all words that have yet to be typeset.
    /// 3. Return the least number of lines. This will be used as a target for all columns to reach.
    pub fn get_min_num_lines(&self) -> Result<Option<usize>, Error> {
        // Get the number of lines per position.
        let mut num_lines = vec![];
        [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .map(|position| self.get_num_lines_tex(position, None))
            .try_for_each(|num| match num {
                Ok(num) => {
                    num_lines.push(num);
                    Ok(())
                }
                Err(error) => Err(error),
            })?;
        match num_lines.into_iter().min() {
            Some(min) => Ok(min),
            None => Err(Error::NoColumns),
        }
    }

    /// Given 1-3 columns and a target `num_lines`, create a TeX table.
    ///
    /// `left`, `center`, and `right` are the columns. At least one of them must be `Empty` or `Text`.
    pub fn get_tex_table(&mut self, num_lines: usize) -> Result<Option<String>, Error> {
        let mut para_columns: [ParaColumn; 3] = Default::default();
        for (position, para_column) in [Position::Left, Position::Center, Position::Right]
            .into_iter()
            .zip(para_columns.iter_mut())
        {
            *para_column = match self.get_cosmic_index(position, num_lines) {
                Some(cosmic_index) => {
                    match self.get_tex_words(position, cosmic_index, num_lines)? {
                        Some(text) => ParaColumn::Text(text),
                        None => ParaColumn::Empty,
                    }
                }
                None => ParaColumn::None,
            };
        }

        if Self::para_columns_done(&para_columns) {
            Ok(None)
        } else {
            Ok(self.get_paracol(&para_columns))
        }
    }

    pub fn done(&self) -> bool {
        [&self.left, &self.center, &self.right]
            .iter()
            .all(|c| c.done())
    }

    /// Convert TeX strings per column into a TeX table.
    fn get_paracol(&self, columns: &[ParaColumn; 3]) -> Option<String> {
        const SWITCH: &str = "\\switchcolumn ";
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
                let mut table = self.begin_paracol.clone();
                // Switch this many time.
                let mut num_switches = num_some - 1;

                // Get the columns with text.
                // Add the text to the table.
                // Add switch statements between the columns (but not at the end).
                for para_column in columns.iter() {
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
                        ParaColumn::None => (),
                    }
                }

                // End the table.
                table.push_str("\n\n\\end{paracol}");
                Some(table)
            }
        }
    }

    /// Get the number of lines in a column using Tectonic.
    ///
    /// - `end` is an optional end index for `self.words`. If `None`, the words are from `self.start..self.words.len()`.
    /// - `width` is the width of the column, as calculated elsewhere.
    /// - `page` is used because we need the preamble.
    fn get_num_lines_tex(
        &self,
        position: Position,
        end: Option<usize>,
    ) -> Result<Option<usize>, Error> {
        let para_columns = match position {
            Position::Left => [
                ParaColumn::new(&self.left, end),
                ParaColumn::new_empty(&self.center),
                ParaColumn::new_empty(&self.right),
            ],
            Position::Center => [
                ParaColumn::new_empty(&self.left),
                ParaColumn::new(&self.center, end),
                ParaColumn::new_empty(&self.right),
            ],
            Position::Right => [
                ParaColumn::new_empty(&self.left),
                ParaColumn::new_empty(&self.center),
                ParaColumn::new(&self.right, end),
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

    /// Typeset using Tectonic to fill a column with `num_lines` of a TeX string.
    /// `cosmic_index` is the best-first-guess of the end index.
    fn get_tex_words(
        &mut self,
        position: Position,
        cosmic_index: usize,
        num_lines: usize,
    ) -> Result<Option<String>, Error> {
        // Get the target column.
        let column = match position {
            Position::Left => &self.left,
            Position::Center => &self.center,
            Position::Right => &self.right,
        };

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
                        end = end.min(len);

                        let column = self.get_mut_column(position);

                        match column {
                            Column::Column { column, width: _ } => {
                                match column {
                                    MaybeSpanColumn::Span(column) => {
                                        if column.start >= end {
                                            None
                                        } else {
                                            let text = column.to_tex(Some(end));
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
                    })
                }
                None => Ok(None),
            }
        }
    }

    fn get_mut_column(&mut self, position: Position) -> &mut Column<'t> {
        match position {
            Position::Left => &mut self.left,
            Position::Center => &mut self.center,
            Position::Right => &mut self.right,
        }
    }

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

        let min_num_lines = table.get_min_num_lines().unwrap().unwrap();
        assert_eq!(min_num_lines, 10);
    }
}
