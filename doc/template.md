# Talmudifier

@OVERVIEW@


```rust
use std::{fs::write, path::PathBuf, str::FromStr};

use talmudifier::prelude::*;

let directory = PathBuf::from_str("example_text").unwrap();

// Load a default talmudifier.
let mut talmudifier = Talmudifier::default()
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

## Getting started

It's easy! You don't need to be a programmer to use Talmudifier.

1. [Download and extract this repo](https://github.com/subalterngames/talmudifier-rs/archive/refs/heads/main.zip)
2. [Download and install Rust](https://www.rust-lang.org/tools/install)
3. Open a terminal shell. 
  - Windows: Search for `powershell` and run it
  - macOS: Search for `terminal` and run it
  - Linux: You know what to do
4. In the terminal, type `cd [PATH]` and press enter. *Do not literally type* `[PATH]`. That's a substitute word for whatever the actual path to the repo is. For example, if you extracted the repo in Downloads, then: `cd ~/Downloads/talmudifier`
5. [Compile as an executable](#compile-as-an-executable)
6. [Create a valid talmudifier.json file](#talmudifierjson)

## Compile as an executable

Talmudifier can be compiled as an executable:

```bash,ignore
cargo build --release --bin talmudify --features clap
```

This will create: `target/release/talmudify`. You can run it like any other shell program: `./talmudify [ARGS]`

```ignore
Usage: talmudify [OPTIONS]

Options:
  -t, --talmudifier <TALMUDIFIER>  The path to a talmudifier json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used [default: talmudifier.json]
  -o, --out <OUT>                  The path to the output directory [default: out]
  -l, --log                        If included, write intermediate .tex and .pdf files to logs/. This is useful for debugging but slow.
  -h, --help                       Print help
  -V, --version                    Print version
```

## talmudifier.json

@CONFIG@

### Length values

@LENGTH@

### Fonts

@FONTS@


### Source text

`"source_text"` specifies the source text that will be talmdufied. There are three options:

1. File paths to three markdown files. These files must exist and must be single paragraphs (no double line breaks):

```ignore
"Files": {
    "left": "left.md",
    "center": "center.md",
    "right": "right.md"
}
```

2. Three markdown strings:

```ignore
"Text": {
    "left": "This is the left column.",
    "center": "This is the center column.",
    "right": "This is the right column."
}
```

3. A single markdown file with exactly three paragraphs:

Example JSON:

```ignore
"File": "text.md"
```

An example file:

```ignore
This is the left column.

This is the center column.

This is the right column.
```

A very subset of markdown is used in Talmudifier:

For the most part, just type text like you normally would. You can italicize text like \*this\*. You can make text bold like \*\*this\*\*. You can make bold and italic text like \*\*\*this\*\*\*. \*\*You can make multiple words bold and you can \*italicize\* within bold text\*\* (\*and \*\*vice\*\* versa\*). \`If you want to add marginalia, use graves.\`

Links, images, headers, emoji, etc. are not supported.

### Title

By default, `"title"` is set to `null`. Set it to something else to add a title to the page:

`"title": "Chapter 1"`

### Logging

Set `"log": true` to enable logging. This will generated intermediary files per iteration that can be useful for debugging. This will also make Talmudifier run slower.

## How it works

@HOW@

## Feature flags

- `default-fonts` embeds default fonts into the executable. You might want to remove this if you want to use other fonts because the default fonts make the binary bigger.
- `ffi` is required by the underlying PDF generator (`tectonic`). *Always include this feature.*
- `clap` is required for some of the executables. If you're using Talmudifier as a library, you can ignore this.

## Changes from Python

This is a Rust port of my `talmudifier` Python module. Major differences include:

- It's much faster.
- No external TeX engine needed. Talmudifier has its own internal TeX engine.
- No need to download any TeX packages. Talmudifier will download the required packages for you.
- Use Cosmic Text to guess the end index that is then used to generate a TeX column of a specified number of lines. Previously, I was using hard-coded guesses to do this. Cosmic Text is much more flexible and accurate.
- Default fonts are automatically included
- Simplified the config file
- No longer supported:
  - 1-1 word/character substitutions
  - Colorization of specific words

## Other executables

To regenerate `example_talmudifier.json`:

```bash,ignore
cargo run --bin example_config
```

To convert an arbitrary .tex file into a .pdf (useful for debugging):

```bash,ignore
cargo run --bin textest --features clap -d directory/ -f filename.tex
```

The `-d` argument is optional and defaults to `logs/`.