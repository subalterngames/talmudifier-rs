# Compile as an executable

Talmudifier can be compiled as an executable:

```bash
cargo build --release --bin talmudify --features clap
```

This will create: `target/release/talmudify`. You can run it like any other shell program: `./talmudify [ARGS]`

```
Usage: talmudify [OPTIONS]

Options:
  -t, --talmudifier <TALMUDIFIER>  The path to a talmudifier json file. If this arg is not included, and if the default-fonts feature is enabled, then default values will be used [default: talmudifier.json]
  -o, --out <OUT>                  The path to the output directory [default: out]
  -l, --log                        If included, write intermediate .tex and .pdf files to logs/. This is useful for debugging but slow.
  -h, --help                       Print help
  -V, --version                    Print version
```

## Other executables

To regenerate `example_talmudifier.json`:

```bash
cargo run --bin example_config
```

To convert an arbitrary .tex file into a .pdf (useful for debugging):

```bash
cargo run --bin textest --features clap -d directory/ -f filename.tex
```

The `-d` argument is optional and defaults to `logs/`.

# TODO

- [x] Uniform font size
- [ ] Test title
- [ ] Inline title
- [x] Remove logging in example
- [ ] Write an actual README
- [ ] Update comments in `table` and `config`
- [ ] Windows
- [ ] MacOS
- [x] Without default features
- [x] Rename config