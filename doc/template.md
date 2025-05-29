# Talmudifier

@OVERVIEW@

```rust
use std::{fs::write, path::PathBuf, str::FromStr};

use talmudifier::prelude::*;

let directory = PathBuf::from_str("example_text").unwrap();

// Load a default talmudifier.
let talmudifier = Talmudifier::default()
    // Add a title to the page.
    .title("Talmudifier")
    // Set the source text as three Markdown files.
    .source_text(SourceText::Files {
        left: directory.join("left.md"),
        center: directory.join("center.md"),
        right: directory.join("right.md")
});

// Talmudify.
let daf = talmudifier.talmudify().unwrap();

// Write the .tex. This is sometimes useful for debugging.
write("out.tex", &daf.tex).unwrap();

// Write the PDF.
write("out.pdf", &daf.pdf).unwrap();
```

## Download talmudifier

On this webpage, there is a **Releases** sidebar. Click that, and download Talmudifier.

Open a terminal. You first need to change directory to Downloads (or wherever Talmudifier actually is), so:

```text
cd ~/Downloads
```

*MacOS:* The app might not be marked as an executable. Run this in the same terminal window:

```text
chmod +x talmudifier
```

*MacOS:* If that didn't work, move `talmudifier` to your home directory, and then run this:

```text
cd ~ && chmod +x talmudifier
```

*MacOS and Linux:* Run Talmudifier:

```text
./talmudifier 
```

*Windows:* Run Talmudifier:

```text
./talmudifier.exe
```

*All platforms:* When you run Talmudifier, you'll see a list of command line options.

You now need to actually write some words for the page, and then [create a valid talmudifier.json file](#talmudifierjson).

Save `talmudifier.json` wherever you want. Assuming that:

- The current directory is your home directory (`/home/<username>/`)
- `talmudify` is in `Downloads/`
- `talmudifier.json` is in `Documents/`
- You want to output the PDF to `Documents/`

...Then you would do this:

```text
./talmudify -t Documents/talmudifier.json -o Documents/out.pdf
```

## Add Talmudifier to your project

The underlying `tectonic` TeX engine uses some C++ libraries which are compiled via vcpkg.

**First time only:**

1. Download and install a C++ compiler
2. Install some required packages:
   - *MacOS:* `brew install autoconf automake autoconf-archive`
   - *Linux (Debian or Ubuntu):* `sudo apt install autoconf automake autoconf-archive`
   - *Linux (something else):* Same packages, different package manager, probably
   - *Windows:* No need to download anything extra
3. Run: `cargo install cargo-vcpkg`

**First time only or whenever you `cargo clean`:**[^1]

```text
cargo vcpkg build
```

**Every time you want to build your project, set the following environment flags:**

*Linux and MacOS:*

```text
export TECTONIC_DEP_BACKEND="vcpkg"
```

*Windows:*

```text
$Env:TECTONIC_DEP_BACKEND="vcpkg"
$Env:RUSTFLAGS="-Ctarget-feature=+crt-static"
```

Then, create a `talmudifier.json` file.

## Compile as an executable

Follow steps for adding Talmudifier to your project. Then, run:

```text
cargo build --release --bin talmudify --features clap
```

Then, create a `talmudifier.json` file.

## talmudifier.json

@CONFIG@

### Length values

@LENGTH@

### Fonts

@FONTS@


### Source text

`"source_text"` specifies the source text that will be talmdufied. There are three options:

1. File paths to three markdown files. These files must exist and must be single paragraphs (no double line breaks):

```text
"Files": {
    "left": "left.md",
    "center": "center.md",
    "right": "right.md"
}
```

2. Three markdown strings:

```text
"Text": {
    "left": "This is the left column.",
    "center": "This is the center column.",
    "right": "This is the right column."
}
```

3. A single markdown file with exactly three paragraphs:

Example JSON:

```text
"File": "text.md"
```

An example file:

```text
This is the left column.

This is the center column.

This is the right column.
```

A very subset of markdown is used in Talmudifier:

@MARKDOWN@

### Title

By default, `"title"` is set to `null`. Set it to something else to add a title to the page:

`"title": "Chapter 1"`

### Logging

Set `"log": true` to enable logging. This will generated intermediary files per iteration that can be useful for debugging. This will also make Talmudifier run slower.

## How it works

@HOW@

## Feature flags

- `default-fonts` embeds default fonts into the executable. You might want to remove this if you want to use other fonts because the default fonts make the binary bigger.
- `clap` is required for some of the executables. If you're using Talmudifier as a library, you can ignore this.
- `textest` is only used for the `textest` binary; it makes some extra functions and structs public.

## Benchmark

To run a very rudimentary benchmark:[^2]

```text
cargo run --bin benchmark --release
```

Current benchmark: 10 seconds

## Other executables

To regenerate `example_talmudifier.json`:

```text
cargo run --bin example_config
```

To convert an arbitrary .tex file into a .pdf (useful for debugging):

```text
cargo run --bin textest --features textest -d directory/ -f filename.tex
```

The `-d` argument is optional and defaults to `logs/`.
You can also, optionally, add `-x` to create a .xdv file instead of a .pdf, which is useful for debugging line counts.

## Changes from Python

This is a Rust port of my `talmudifier` Python module. Major differences include:

- It's 21.6 times faster.[^3]
- No external TeX engine needed. Talmudifier has its own internal TeX engine.
- No need to manually download any TeX packages. Talmudifier will download the required packages for you.
- Two major performance improvements to the *algorithm*:
  - Python Talmudifier uses hard-coded values to guess the maximum number of words that can fit in a cell, and then uses that guess as the start index for finding the actual number. Rust Talmudifier also guesses the start index, but uses Cosmic Text, which is more flexible and accurate.
  - When counting lines, Python Talmudifier extracted text from a pdf that was saved to disk. Rust Talmudifier parses a .xdv file in-memory.
  - When trying to fill a cell with words, Python Talmudifier increments or decrements one word at a time. This always works, but there is overhead to rendering many single pages vs. a single multi-page render. Rust Talmudifier renders multiple pages of incrementing/decrementing guesses. The resulting process is roughly four times faster than it would've been if Rust Talmudifier rendered separate PDFs.

- Default fonts are embedded in the executable
- Simplified the config file
- No longer supported:
  - 1-1 word/character substitutions
  - Colorization of specific words

***

[^1]: If `cargo vcpkg build` fails, it's probably because you've got a whitespace in the root file path. To fix: Move `target/vcpkg` to a directory without white spaces, such as: `C:/vcpkg`. Then: `cd C:/vcpkg` Then: `vcpkg install icu` on MacOS and Linux, or `vcpkg install icu --triplet x64-windows-static` on Windows. Then: Move `vcpkg/` back to `<project>/target/` The Internet implies that a newer compiler than what I'm using might fix the problem. Or maybe it won't. Sorry.
[^2]: There's no need for anything more complicated than this because Talmudifier is so slow.
[^3]: See the benchmark. With Python Talmudifier, a similar benchmark takes 216 seconds.
