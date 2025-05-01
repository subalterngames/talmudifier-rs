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

## If you're not a programmer

**If you're not a programmer, I promise that you're capable of installing and using Talmudifier. If you run into trouble, please [contact me](mailto:subalterngames@gmail.com).**

1. Download this repo. There's a green `<> Code` button on this page. Click it, download a .zip file, and extract the zip file.
2. [Install Rust](https://www.rust-lang.org/tools/install)
3. [Install git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
4. [Compile the C++ libraries](#compile-the-c++-libraries)
5. [Compile as an executable](#compile-as-an-executable)
6. [Create a valid talmudifier.json file](#talmudifierjson)

```text
Whenever you see text like this, it means: 
1. Open a terminal
2. Type in the command
3. Press enter
```

Most of the commands require your *current directory* to be `talmudifier-rs`. We need to *cd* to the *current directory.*

Let's say you downloaded Talmudifier and now it's in Downloads. You need to do this:

```text
cd ~/Downloads/talmudifier-rs
```

You will need to run this command every time you open a new terminal window.

## Compile the C++ libraries

The underlying `tectonic` TeX engine uses some C++ libraries, so compiling requires a few more steps than `cargo build`.

On Debian,  you can run `./tectonic.sh` to install the C++ libraries, and skip steps 1-4. I haven't tested this script on other distros but you can make your own version as needed.

If you don't want to install system packages, or if you're not using Linux, then you're going to compile via vcpkg[^1]:

1. Download and install a C++ compiler

2. Run:

   ```text
   cargo install cargo-vcpkg
   ```

3. Run:[^2]

   ```text
    cargo vcpkg build
   ```

4. Set environment flags (do this every time you open a new terminal window):

   - *Linux and MacOS:* 
     
     ```text
     export TECTONIC_DEP_BACKEND="vcpkg"
     ```
     
   - *Windows:* 

     ```text
     $Env:TECTONIC_DEP_BACKEND="vcpkg"
     $Env:RUSTFLAGS="-Ctarget-feature=+crt-static"
     ```

## Compile as an executable

```text
cargo build --release --bin talmudify --features clap
```

The compiled executable is in `target/release/`. To get a list of options, run:

```text
./talmudifier --help
```

On Windows, it's `./talmudifier.exe` instead of `./talmudifier`

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

## Benchmark

To run a very rudimentary benchmark:[^3]

```text
cargo run --bin benchmark --release
```

Current benchmark: 35 seconds

## Other executables

To regenerate `example_talmudifier.json`:

```text
cargo run --bin example_config
```

To convert an arbitrary .tex file into a .pdf (useful for debugging):

```text
cargo run --bin textest --features clap -d directory/ -f filename.tex
```

The `-d` argument is optional and defaults to `logs/`.

## Changes from Python

This is a Rust port of my `talmudifier` Python module. Major differences include:

- It's over six times faster.[^4]
- No external TeX engine needed. Talmudifier has its own internal TeX engine.
- No need to manually download any TeX packages. Talmudifier will download the required packages for you.
- Two major performance improvements to the *algorithm*:
  - Python Talmudifier uses hard-coded values to guess the maximum number of words that can fit in a cell, and then uses that guess as the start index for finding the actual number. Rust Talmudifier also guesses the start index, but uses Cosmic Text, a crate normally used for GUI text, to dynamically calculate the guess. Because the Cosmic Text guess canvary depending on font parameters, Rust Talmudifier's guess is more flexible and more accurate.
  - When trying to fill a cell with words, Python Talmudifier increments or decrements one word at a time. This always works, but there is overhead to rendering many single pages vs. a single multi-page render. Rust Talmudifier renders multiple pages of incrementing/decrementing guesses. The resulting process is roughly 2.5 times faster than it would've been if Rust Talmudifier rendered separate PDFs.

- Default fonts are embedded in the executable
- Simplified the config file
- No longer supported:
  - 1-1 word/character substitutions
  - Colorization of specific words

***

[^1]: Tested on Windows, but vcpkg should work the same on Linux and MacOS.
[^2]: If `cargo vcpkg build` fails when trying to compile `icu`, it's probably because you've got a whitespace in the root file path. To fix: Move `target/vcpkg` to a directory without white spaces, such as: `C:/vcpkg`. Then: `cd C:/vcpkg`/ Then: `vcpkg install icu --triplet x64-windows-static`. Then: Move `vcpkg/` back to `<project>/target/` The Internet implies that a newer compiler than what I'm using might fix the problem. Or maybe it won't. Sorry.
[^3]: There's no need for anything more complicated than this because Talmudifier is so slow.
[^4]: See the benchmark. With Python Talmudifier, a similar benchmark takes 216 seconds.