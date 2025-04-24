You can also optionally set the per-column `"fonts"`. If you don't, and if you've included the `default-features` feature, default fonts will be used. See `example_fonts.json`; you can replace `null` with the text in `example_fonts.json` (assuming that the files actually exist).

Limitations:

- Each font style (regular, bold, etc.) *must* be a separate file.
- A column's font files must all be in the same directory.
- System fonts are not supported.