`"fonts"` has a default value of `null`, in which case default fonts embedded in the executable are used. You can set this to use other fonts. See `example_fonts.json`; you can replace `null` with the text in `example_fonts.json` (assuming that the files actually exist).

Limitations:

- Each font style (regular, bold, etc.) *must* be a separate file.
- A column's font files must all be in the same directory.
- System fonts are not supported.