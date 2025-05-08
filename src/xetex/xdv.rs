//! XDV is an intermediary file format in XeTeX.
//!
//! DVI is an ancient file format that no one uses anymore except by pdflatex, internally.
//! A .tex is converted into a .dvi file, which is converted into a .pdf file.
//! This may sound cumbersome, but a) it is and b) it's OK because DVI is simple and fast to create.
//!
//! XDV is an Xtension of DVI for XeTeX. So: .tex -> .xdv -> .pdf
//! This file doesn't attempt to actually parse an XDV file because we don't need to.
//! Talmudifier just wants to know the line counts per page.
//! Checking the line counts of the XDV file is much faster than checking the line counts of the final PDF file.
//!
//! I learned out how to do this from reading the code of dvi (Rust) and dvisvgm (C++)
//! https://github.com/mgieseki/dvisvgm/
//! https://github.com/richard-uk1/dvi-rs/

use nom::{
    bytes::streaming::tag,
    error::ErrorKind,
    number::streaming::{be_u16, be_u24, be_u32, be_u8},
};
use tectonic::{
    config::PersistentConfig,
    ctry,
    driver::{OutputFormat, ProcessingSessionBuilder},
    errmsg,
    status::NoopStatusBackend,
};

use crate::error::Error;

#[derive(Copy, Clone, Eq, PartialEq)]
enum DviVersion {
    Dvi,
    Xdv5,
    Xdv6,
    Xdv7,
}

/// A partial .xdv parser.
struct Xdv<'t> {
    data: &'t [u8],
    version: DviVersion,
}

impl<'t> Xdv<'t> {
    fn new(data: &'t Vec<u8>) -> Self {
        // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/BasicDVIReader.hpp#L50
        const OP_PRE: u8 = 247;

        let mut xdv = Self {
            data,
            version: DviVersion::Dvi,
        };

        // Parse the preamble.
        if xdv.read_u8() == OP_PRE {
            // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L180
            xdv.version = match xdv.read_u8() {
                0..5 => DviVersion::Dvi,
                5 => DviVersion::Xdv5,
                6 => DviVersion::Xdv6,
                7 => DviVersion::Xdv7,
                other => unreachable!("Invalid version {}", other),
            };

            // num[4] + dem[4] + mag[4]
            xdv.advance(12);

            // Comment.
            let k = xdv.read_u8() as usize;
            xdv.advance(k);
        }

        xdv
    }

    /// Count the number of line breaks, separated by page breaks.
    fn get_num_lines(mut self) -> Vec<usize> {
        let mut num_lines_per_page = vec![];
        let mut num_lines = 1;
        let mut got_words = false;
        while self.data.len() > 1 {
            // https://github.com/richard-uk1/dvi-rs/blob/c8078c37065fe7b72b09586c10ee220a7c91d99b/src/parser.rs#L12
            // Get the op code.
            let op = self.read_u8();
            match op {
                // Set
                0..=127 => (),
                // Char and move right
                128..=131 => self.advance4(op, 131),
                // SetRule
                132 => self.advance(8),
                // Put char
                133..=136 => self.advance4(op, 136),
                // Rule
                137 => self.advance(8),
                // Nop
                138 => (),
                // Bop
                139 => self.advance(44),
                // Eop
                140 => {
                    num_lines_per_page.push(num_lines);
                    num_lines = 1;
                    got_words = false;
                }
                // Push and Pop
                141 | 142 => (),
                // Right
                143..=146 => self.advance4(op, 146),
                // RightBy and set W
                147..=151 => self.advance4(op, 151),
                // RightBy and set X
                152..=156 => self.advance4(op, 156),
                // Down
                157..=160 => self.advance4(op, 160),
                // Down and set Y
                161..=165 => {
                    num_lines += 1;
                    self.advance4(op, 165)
                }
                // Down and set Z
                166..=170 => {
                    num_lines += 1;
                    self.advance4(op, 170)
                }
                // SetFont to i
                171..=234 => (),
                // SetFont
                235..=238 => self.advance4(op, 238),
                // Xxx
                239 => {
                    let k = self.read_u8() as usize;
                    self.advance(k);
                }
                240 => {
                    let k = self.read_u16() as usize;
                    self.advance(k);
                }
                241 => {
                    let k = self.read_u24() as usize;
                    self.advance(k);
                }
                242 => {
                    let k = self.read_u32() as usize;
                    self.advance(k);
                }
                // FontDef
                243 => self.advance(1),
                // FontNumberDef
                244..=246 => self.advance4(op, 246),
                // Pre
                247 => {
                    // i[1] + num[4] + den[4] + mag[4]
                    self.advance(13);
                    // k[1] + x[k]
                    let k = self.read_u8() as usize;
                    self.advance(k);
                }
                // Post
                248 => self.advance(28),
                // PostPost
                249 => {
                    self.advance(5);
                    self.parse_223()
                }
                // ???
                250 => unreachable!("Code 250 is never used."),
                // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L578
                251 => unreachable!("Code 251 is pic but there are none in Talmudifier."),
                // Font def (XeTeX).
                // See: https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L591
                252 => {
                    // fontnum[4] + ptsize[4]
                    self.advance(8);
                    let flags = self.read_u16();
                    let psname_len = self.read_u8() as usize;
                    let fname_len = if self.version == DviVersion::Xdv5 {
                        self.read_u8() as usize
                    } else {
                        0
                    };
                    let stname_len = if self.version == DviVersion::Xdv5 {
                        self.read_u8() as usize
                    } else {
                        0
                    };
                    // Font name.
                    self.advance(psname_len);

                    if self.version == DviVersion::Xdv5 {
                        self.advance(fname_len + stname_len);
                    } else {
                        self.advance(4);
                    }

                    // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L606-L624
                    [0x0200, 0x1000, 0x2000, 0x4000].into_iter().for_each(|f| {
                        if (flags & f) != 0 {
                            self.advance(4);
                        }
                    });

                    if self.version == DviVersion::Xdv5 && ((flags & 0x0800) != 0) {
                        let num_variations = self.read_i16() as usize;
                        self.advance(num_variations * 4);
                    }
                }
                // Glyph IDs and positions.
                // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L653
                253 => {
                    if self.version == DviVersion::Xdv7 {
                        got_words = true;
                        // w[4]
                        self.advance(4);
                        let n = self.read_u16();
                        // dx[4n] + dy[4] + glypns[2n]
                        self.advance(10 * n as usize);
                    }
                }
                // UTF-8, Y positions are always the same.
                // https://github.com/mgieseki/dvisvgm/blob/ef6a9e03e72a46a41bede2d406810e0e9b9fab61/src/DVIReader.cpp#L671
                254 => {
                    if self.version == DviVersion::Xdv5 {
                        got_words = true;
                        // l[2]
                        let l = self.read_u16();
                        // chars[2 * l]
                        self.advance(2 * l as usize);
                        // w[4]
                        self.advance(4);
                        // n[2]
                        let n = self.read_u16();
                        // (dx,dy)[(4+4)n] glyphs[2n]
                        self.advance(6 * n as usize + 4);
                    }
                }
                255 => (),
            };
        }
        if got_words {
            num_lines_per_page.push(num_lines);
        }
        num_lines_per_page
    }

    fn read_u8(&mut self) -> u8 {
        let (x, v) = be_u8::<(_, ErrorKind)>(self.data).unwrap();
        self.data = x;
        v
    }

    fn read_u16(&mut self) -> u16 {
        let (x, v) = be_u16::<(_, ErrorKind)>(self.data).unwrap();
        self.data = x;
        v
    }

    fn read_i16(&mut self) -> u16 {
        let (x, v) = be_u16::<(_, ErrorKind)>(self.data).unwrap();
        self.data = x;
        v
    }

    fn read_u24(&mut self) -> u32 {
        let (x, v) = be_u24::<(_, ErrorKind)>(self.data).unwrap();
        self.data = x;
        v
    }

    fn read_u32(&mut self) -> u32 {
        let (x, v) = be_u32::<(_, ErrorKind)>(self.data).unwrap();
        self.data = x;
        v
    }

    fn advance(&mut self, delta: usize) {
        self.data = &self.data[delta..];
    }

    fn advance4(&mut self, op: u8, max: u8) {
        self.advance(4 - (max - op) as usize)
    }

    fn parse_223(&mut self) {
        while let Ok((inext, _)) = tag::<_, _, ()>(&[223][..])(self.data) {
            self.data = inext;
        }
    }
}

/// This is mostly copied from tectonic's latex_to_pdf
fn latex_to_xdv<T: AsRef<str>>(latex: T) -> tectonic::Result<Vec<u8>> {
    let mut status = NoopStatusBackend::default();

    let auto_create_config_file = false;
    let config = ctry!(PersistentConfig::open(auto_create_config_file);
                       "failed to open the default configuration file");

    let only_cached = false;
    let bundle = ctry!(config.default_bundle(only_cached, &mut status);
                       "failed to load the default resource bundle");

    let format_cache_path = ctry!(config.format_cache_path();
                                  "failed to set up the format cache");

    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sb = ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(OutputFormat::Xdv)
            .do_not_write_output_files();

        let mut sess =
            ctry!(sb.create(&mut status); "failed to initialize the LaTeX processing session");
        ctry!(sess.run(&mut status); "the LaTeX engine failed");
        sess.into_file_data()
    };

    match files.remove("texput.xdv") {
        Some(file) => Ok(file.data),
        None => Err(errmsg!(
            "LaTeX didn't report failure, but no PDF was created (??)"
        )),
    }
}

pub fn get_num_lines<T: AsRef<str>>(latex: T) -> Result<Vec<usize>, Error> {
    match latex_to_xdv(latex) {
        Ok(data) => Ok(Xdv::new(&data).get_num_lines()),
        Err(error) => Err(Error::Xdv(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::Xdv;

    #[test]
    fn test_xdv() {
        let data = include_bytes!("../../test_text/out.xdv").to_vec();
        let xdv = Xdv::new(&data);
        let lines = xdv.get_num_lines();
        assert_eq!(lines.len(), 20);
        [4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7]
            .into_iter()
            .enumerate()
            .for_each(|(i, n)| {
                assert_eq!(lines[i], n);
            });
    }
}
