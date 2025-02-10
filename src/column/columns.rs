use super::{column_type::ColumnType, position::Position};

/// Enum values used to describe the columns that we currently want to typeset.
#[derive(Debug)]
pub enum Columns {
    One,
    LeftRight,
    LeftCenter,
    CenterRight,
    Three,
}

impl Columns {
    const ONE_HALF: f32 = 0.5;
    const TWO_THIRDS: f32 = 0.675;
    const ONE_THIRD: f32 = 0.32;
    const END: &'static str = "\n\n\\end{paracol}";
    const SWITCH_COLUMN: &'static str = "\\switchcolumn";
    const SWITCH_COLUMN_2: &'static str = "\\switchcolumn[2]";

    /// Returns the width of the column as a fraction of the page width.
    pub fn get_width(&self, position: &Position) -> f32 {
        match (self, position) {
            (Self::One, _) => 1.,
            (Self::LeftRight, Position::Left) | (Self::LeftRight, Position::Right) => {
                Columns::ONE_HALF
            }
            (Self::LeftCenter, Position::Left) => Columns::ONE_THIRD,
            (Self::LeftCenter, Position::Center) => Columns::TWO_THIRDS,
            (Self::CenterRight, Position::Center) => Columns::TWO_THIRDS,
            (Self::CenterRight, Position::Right) => Columns::ONE_THIRD,
            (Self::Three, _) => Columns::ONE_THIRD,
            (a, b) => panic!("Invalid position: {:?} {:?}", a, b),
        }
    }

    /// Given three columns (some of which are potentially non-existent), create a TeX string.
    /// This string is the table with the appropriate number of columns, and with text in each column that needs it.
    pub fn get_columns(left: ColumnType, center: ColumnType, right: ColumnType) -> String {
        let mut tex = String::new();
        match (left, center, right) {
            // Left, Right, Center.
            (ColumnType::Text(left), ColumnType::Text(center), ColumnType::Text(right)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(&left);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // Left, Center, Empty.
            (ColumnType::Text(left), ColumnType::Text(center), ColumnType::Empty) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(&left);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
            }
            // Left, Empty, Right.
            (ColumnType::Text(left), ColumnType::Empty, ColumnType::Text(right)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(&left);
                tex.push_str(Self::SWITCH_COLUMN_2);
                tex.push_str(&right);
            }
            // Empty, Center, Right.
            (ColumnType::Empty, ColumnType::Text(center), ColumnType::Text(right)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // Left, Center, None.
            (ColumnType::Text(left), ColumnType::Text(center), ColumnType::None) => {
                tex.push_str(&Self::LeftCenter.get_latex_header());
                tex.push_str(&left);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
            }
            // Left, None, Right.
            (ColumnType::Text(left), ColumnType::None, ColumnType::Text(right)) => {
                tex.push_str(&Self::LeftRight.get_latex_header());
                tex.push_str(&left);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // None, Center, Right.
            (ColumnType::None, ColumnType::Text(center), ColumnType::Text(right)) => {
                tex.push_str(&Self::CenterRight.get_latex_header());
                tex.push_str(&center);
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // Left, Empty, Empty.
            (ColumnType::Text(left), ColumnType::Empty, ColumnType::Empty) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(&left);
            }
            // Empty, Center, Empty.
            (ColumnType::Empty, ColumnType::Text(center), ColumnType::Empty) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
            }
            // Empty, Empty, Left.
            (ColumnType::Empty, ColumnType::Empty, ColumnType::Text(left)) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN_2);
                tex.push_str(&left);
            }
            // Left, Empty, None.
            (ColumnType::Text(left), ColumnType::Empty, ColumnType::None) => {
                tex.push_str(&Self::LeftCenter.get_latex_header());
                tex.push_str(&left);
            }
            // Empty, Center, None.
            (ColumnType::Empty, ColumnType::Text(center), ColumnType::None) => {
                tex.push_str(&Self::LeftCenter.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
            }
            // Empty, None, Right.
            (ColumnType::Empty, ColumnType::None, ColumnType::Text(right)) => {
                tex.push_str(&Self::LeftRight.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // Left, None, Empty.
            (ColumnType::Text(left), ColumnType::None, ColumnType::Empty) => {
                tex.push_str(&Self::Three.get_latex_header());
                tex.push_str(&left);
            }
            // None, Center, Empty.
            (ColumnType::None, ColumnType::Text(center), ColumnType::Empty) => {
                tex.push_str(&Self::LeftCenter.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&center);
            }
            // None, Empty, Right.
            (ColumnType::None, ColumnType::Empty, ColumnType::Text(right)) => {
                tex.push_str(&Self::CenterRight.get_latex_header());
                tex.push_str(Self::SWITCH_COLUMN);
                tex.push_str(&right);
            }
            // Left, None, None.
            // None, Center, None,
            // None, None, Right.
            (ColumnType::Text(text), ColumnType::None, ColumnType::None)
            | (ColumnType::None, ColumnType::Text(text), ColumnType::None)
            | (ColumnType::None, ColumnType::None, ColumnType::Text(text)) => {
                tex.push_str(&Self::One.get_latex_header());
                tex.push_str(&text);
            }
            // All empty and/or none.
            _ => {
                tex.push_str(&Self::One.get_latex_header());
            }
        }
        tex.push_str(Self::END);
        tex
    }

    /// Returns the start of a paracol.
    fn get_latex_header(&self) -> String {
        format!(
            "\\columnratio{{{}}}\n\\begin{{paracol}}{{{}}}\n",
            match self {
                Self::LeftRight => format!("{},{}", Self::ONE_HALF, Self::ONE_HALF),
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
