/// The position of a column on the page.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Position {
    Left,
    Center,
    Right,
}

pub const POSITIONS: [Position; 3] = [Position::Left, Position::Center, Position::Right];
