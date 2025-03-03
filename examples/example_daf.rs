use std::{fs::write, path::PathBuf, str::FromStr};

use talmudifier::prelude::*;

fn main() {
    let daf = Config::default()
        .source_text(SourceText::File(
            PathBuf::from_str("test_text/test.md").unwrap(),
        ))
        .talmudify()
        .unwrap();
    write("out.pdf", daf.pdf).unwrap();
}
