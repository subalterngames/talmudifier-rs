#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub enum Style {
    #[default]
    Regular,
    Italic,
    Bold,
    BoldItalic,
}
