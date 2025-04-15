//! Generate `example_talmudifier.json`

#[cfg(feature = "default-fonts")]
fn main() {
    use std::fs::write;

    use serde_json::to_string_pretty;
    use talmudifier::prelude::*;

    let talmudifier = Talmudifier::default();
    let s = to_string_pretty(&talmudifier).unwrap();
    write("example_talmudifier.json", s).unwrap();

    let fonts = Fonts::default();
    let s = to_string_pretty(&fonts).unwrap();
    write("example_fonts.json", s).unwrap();
}

#[cfg(not(feature = "default-fonts"))]
fn main() {}
