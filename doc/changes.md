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