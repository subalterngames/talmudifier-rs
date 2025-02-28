mod column;
mod config;
mod error;
pub(crate) mod font;
pub(crate) mod page;
pub mod prelude;
pub(crate) mod word;

/// Short hand for simple TeX commands.
/// Example input: `tex!("begin", "document")`
/// Example output: `\begin{document}`
#[macro_export]
macro_rules! tex {
    ($command:expr, $($value:expr),+) => {
        {
            let mut t = format!("\\{}", &$command);
            $(
                t.push_str(&format!("{{{}}}", &$value));
            )+
            t
        }
    };
}

#[cfg(test)]
mod tests {
    use tectonic::latex_to_pdf;

    pub(crate) fn get_test_md() -> (String, String, String) {
        let raw = include_str!("../test_text/test.md")
            .split("\n\n")
            .collect::<Vec<&str>>();
        assert_eq!(raw.len(), 3);
        (raw[0].to_string(), raw[1].to_string(), raw[2].to_string())
    }

    #[test]
    fn test_tex() {
        for (tex, path) in [
            include_str!("../test_text/hello_world.tex"),
            include_str!("../test_text/minimal_daf.tex"),
            include_str!("../test_text/paracol.tex"),
            include_str!("../test_text/daf.tex"),
        ]
        .iter()
        .zip(["hello_world", "minimal_daf", "paracol", "daf"])
        {
            if let Err(error) = latex_to_pdf(tex.replace("\r", "")) {
                panic!("Tex error: {} {}", error, path)
            }
        }
    }
}
