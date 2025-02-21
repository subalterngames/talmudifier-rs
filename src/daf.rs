use crate::{column::Column, page::Page};

pub struct Daf {
    pub left: Column,
    pub center: Column,
    pub right: Column,
    pub page: Page,
    pub title: Option<String>,
}
