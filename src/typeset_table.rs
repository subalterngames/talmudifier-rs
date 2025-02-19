use crate::{tex::table::Table, word::Word};

pub struct TypesetTable<'t> {
    pub tex: String,
    pub table: Table,
    pub left: &'t [Word],
    pub center: &'t [Word],
    pub right: &'t [Word],
}