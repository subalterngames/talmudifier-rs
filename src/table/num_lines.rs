use crate::error::Error;

use super::position::Position;

pub struct NumLines {
    pub left: Option<usize>,
    pub center: Option<usize>,
    pub right: Option<usize>,
}

impl NumLines {
    pub fn get_min(&self) -> Result<(Option<usize>, Position), Error> {
        match [self.left, self.center, self.right]
            .into_iter()
            .zip([Position::Left, Position::Center, Position::Right])
            .min_by(|a, b| a.0.cmp(&b.0))
        {
            Some(min) => Ok((min.0, min.1)),
            None => Err(Error::NoColumns),
        }
    }
}
