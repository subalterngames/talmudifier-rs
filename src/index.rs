/// Indices used to traverse spans of words.
#[derive(Default, Clone)]
pub struct Index {
    /// The index in a vec of spans.
    pub span: usize,
    /// The index in the span's words.
    pub word: usize,
    /// The index in a hyphenated word.
    pub hyphen: Option<usize>,
}
