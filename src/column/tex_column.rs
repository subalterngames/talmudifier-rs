use crate::tex;

use super::width::Width;

/// A TeX string and a column width.
pub struct TexColumn {
    /// The width of the column.
    pub width: Width,
    /// If None, the input column didn't have words that we could typeset.
    pub text: Option<String>,
}

impl TexColumn {
    /// Convert a slice of columns into a TeX table.
    pub fn get_table(columns: &[Self]) -> String {
        if columns.is_empty() {
            return String::default();
        }
        // Get the TeX header.
        let mut table = tex!(
            "columnratio",
            columns
                .iter()
                .map(|c| c.width.column_ratio().to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        table.push('\n');
        table.push_str(&tex!("begin", "paracol", columns.len()));
        table.push('\n');
        // Add the text.
        for column in columns.iter() {
            // Add some text.
            if let Some(text) = &column.text {
                table.push_str(text);
            }
            // Switch columns.
            table.push_str("\\switchcolumn ");
        }
        // End the table.
        table.push_str("\n\n\\end{paracol}");
        table
    }
}
