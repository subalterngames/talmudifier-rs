use super::width::Width;

/// This is used to determine which columns to include when typesetting.
pub struct TexColumn {
    pub width: Width,
    pub text: Option<String>,
}
