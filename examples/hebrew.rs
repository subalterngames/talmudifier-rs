use std::fs::write;

use talmudifier::prelude::*;

fn main() {
    let daf = Talmudifier::new("examples/hebrew/config.json")
        .unwrap()
        .talmudify()
        .unwrap();
    write("out.pdf", daf.pdf).unwrap();
}
