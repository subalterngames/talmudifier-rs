use crate::tex;

use super::width::Width;

/// This is used to determine which columns to include when typesetting.
pub struct TexColumn {
    pub width: Width,
    pub text: Option<String>,
}

impl TexColumn {
    pub fn get_table(columns: &[Self]) -> String {
        // Get the TeX header.
        let mut table = tex!(
            "columnratio",
            columns
                .iter()
                .map(|c| c.width.column_ratio().to_string())
                .collect::<Vec<String>>()
                .join(",")
        );
        table.push_str(&tex!("begin", "paracol", columns.len()));
        table.push('\n');
        // Add the text.
        let mut tex_strings = vec![];
        for column in columns.iter() {
            if let Some(text) = &column.text {
                tex_strings.push(text.clone());
            }
        }
        table.push_str(&tex_strings.join("\\switchcolumn "));
        // End the table.
        table.push_str("\n\n\\end{paracol}");
        table
    }
}
