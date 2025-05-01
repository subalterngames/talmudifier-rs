#[cfg(feature = "default-fonts")]
use std::{path::PathBuf, str::FromStr, time::Instant};
#[cfg(feature = "default-fonts")]
use talmudifier::prelude::*;

/// We're not using a proper benchmarking crate because Talmudifier is slow.
/// We don't want or need multiple iterations.  
#[cfg(feature = "default-fonts")]
fn main() {
    let directory = PathBuf::from_str("example_text").unwrap();

    // Load a default talmudifier.
    let talmudifier = Talmudifier::default()
        // Add a title to the page.
        .title("Talmudifier")
        // Set the source text as three Markdown files.
        .source_text(SourceText::Files {
            left: directory.join("left.md"),
            center: directory.join("center.md"),
            right: directory.join("right.md"),
        });

    // Talmudify.
    let t0 = Instant::now();
    let _ = talmudifier.talmudify();
    println!("{} seconds", (Instant::now() - t0).as_secs());
}

#[cfg(not(feature = "default-fonts"))]
fn main() {}