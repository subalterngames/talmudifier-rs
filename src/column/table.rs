use crate::tex;

use super::tex_column::TexColumn;

pub fn get_table(columns: &[TexColumn]) -> String {
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
