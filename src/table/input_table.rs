use crate::{column::Column, error::Error, word::Word};

pub enum InputTable {
    One(Column),
    LeftRight {
        left: Column,
        right: Column,
    },
    LeftCenter {
        left: Column,
        center: Column,
    },
    CenterRight {
        center: Column,
        right: Column,
    },
    Three {
        left: Column,
        center: Column,
        right: Column,
    },
}

impl InputTable {
    pub fn get_table(&self,
        num_lines: Option<usize>,
    ) -> Result<(Self, String), Error> {
        // Get the target number of lines.
        let num_lines = match num_lines {
            // Use a hardcoded number of lines.
            Some(num_lines) => num_lines,
            // Get the minimum number of lines.
            None => Column::get_min_num_lines(left, center, right, table)?,
        };
        let column_words = [left, center, right].par_iter_mut().map(|column| {
            let q = column.get_tex_column(num_lines, width);
        });
    }

    pub fn get_min_num_lines(&mut self
    ) -> Result<usize, Error> {
        let has_words = [&left, &center, &right]
            .iter()
            .map(|c| !c.words[c.start..].is_empty())
            .collect::<Vec<bool>>();
        // Get the column with the least words.
        let num_lines = [left, center, right]
            .into_par_iter()
            .zip(
                [Position::Left, Position::Center, Position::Right]
                    .into_par_iter()
                    .zip(has_words.into_par_iter()),
            )
            .filter_map(|(w, (p, h))| {
                if !h {
                    None
                } else {
                    let width = table.get_width(&p);
                    Some(w.get_num_lines_tex(None, width))
                }
            })
            .collect::<Vec<Result<usize, Error>>>();
        if let Some(error) = num_lines.iter().find_map(|n| match n {
            Ok(_) => None,
            Err(error) => Some(error),
        }) {
            Err(Error::MinNumLines(error.to_string()))
        } else {
            match num_lines
                .iter()
                .filter_map(|n| match n {
                    Ok(n) => Some(n),
                    Err(_) => None,
                })
                .min()
            {
                Some(min) => Ok(*min),
                None => Err(Error::NoMinNumLines),
            }
        }
    }
}