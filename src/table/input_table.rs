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
}