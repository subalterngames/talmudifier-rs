use serde::{Deserialize, Serialize};

use crate::page::Page;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub page: Page
}