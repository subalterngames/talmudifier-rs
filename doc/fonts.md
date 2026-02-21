You can also optionally set the per-column `"fonts"`. If you don't, and if you've included the `default-features` feature, default fonts will be used.

Limitations:

- Each font style (regular, bold, etc.) *must* be a separate file.
- A column's font files must all be in the same directory.
- System fonts are not supported.

**You can optionally set the font's language.** For example to set the font to Hebrew, add: `"language": "hebrew"`. By default, the font is `"english"`. For a list of possible languages, [read this](https://texdoc.org/serve/polyglossia/0). 