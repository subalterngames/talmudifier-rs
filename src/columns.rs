pub enum Columns {
    Header,
    One,
    LeftCenter,
    CenterRight,
    Three,
}

impl Columns {
    const ONE_HALF: &'static str = "0.5";
    const TWO_THIRDS: &'static str = "0.675";
    const ONE_THIRD: &'static str = "0.32";
    const END: &'static str = "\n\n\\end{paracol}";

    pub fn get_columns(left: Option<&str>, center: Option<&str>, right: Option<&str>) -> String {
        let mut tex = String::new();
        match (left, center, right) {
            (Some(left), Some(center), Some(right)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(left);
                tex.push_str("\\switchcolumn");
                tex.push_str(center);
                tex.push_str("\\switchcolumn");
                tex.push_str(right);
            }
            (Some(left), Some(center), None) => {
                tex.push_str(&Self::LeftCenter.get_latex_header());
                tex.push_str(left);
                tex.push_str("\\switchcolumn");
                tex.push_str(center);
            }
            (None, Some(center), Some(right)) => {
                tex.push_str(&Self::CenterRight.get_latex_header());
                tex.push_str(center);
                tex.push_str("\\switchcolumn");
                tex.push_str(right);
            }
            (Some(left), None, Some(right)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(left);
                tex.push_str("\\switchcolumn[2]");
                tex.push_str(right);
            }
            (Some(text), None, None) | (None, Some(text), None) | (None, None, Some(text)) => {
                tex.push_str(&Self::One.get_latex_header());
                tex.push_str(text);
            }
            (None, None, None) => {
                tex.push_str(&Self::One.get_latex_header());
            }
        }
        tex.push_str(Self::END);
        tex
    }

    fn get_latex_header(&self) -> String {
        format!(
            "\\columnratio{{{}}}\n\\begin{{paracol}}{{{}}}\n",
            match self {
                Self::Header => format!("{},{}", Self::ONE_HALF, Self::ONE_HALF),
                Self::One => "1".to_string(),
                Self::Three => format!(
                    "{},{},{}",
                    Self::ONE_THIRD,
                    Self::ONE_THIRD,
                    Self::ONE_THIRD
                ),
                Self::LeftCenter => format!("{},{}", Self::ONE_THIRD, Self::TWO_THIRDS),
                Self::CenterRight => format!("{},{}", Self::TWO_THIRDS, Self::ONE_THIRD),
            },
            match self {
                Self::One => 1,
                Self::Three => 3,
                _ => 2,
            }
        )
    }
}
