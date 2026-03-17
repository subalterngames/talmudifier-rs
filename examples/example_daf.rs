use std::{fs::write, path::PathBuf, str::FromStr};

use talmudifier::prelude::*;

fn main() {
    let daf = Talmudifier::default()
        .source_text(SourceText::File(
            PathBuf::from_str("test_text/test.md").unwrap(),
        ))
        .log()
        .talmudify()
        .unwrap();
    write("out.pdf", daf.pdf).unwrap();
}
