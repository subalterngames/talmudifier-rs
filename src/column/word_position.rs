#[derive(Copy, Clone, Default, Eq, PartialEq, Debug)]
pub enum WordPosition {
    #[default]
    Body,
    Footnote,
    Title,
}
