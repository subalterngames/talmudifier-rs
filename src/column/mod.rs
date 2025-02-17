use crate::word::Word;

use error::Error;

pub mod cosmic;
pub mod tex;
pub mod width;

pub trait ColumnMaker {
    fn get_num_lines(&mut self, words: &[Word]) -> Result<usize, Error>;

    fn get_words(&mut self, words: &[Word], num_lines: usize) -> Result<Option<usize>, Error> {
        if num_lines > 0 {
            for i in 0..words.len() {
                // We exceeded the number of lines. Break!
                match self.get_num_lines(&words[..i]) {
                    Ok(num) => {
                        if num > num_lines {
                            return Ok(if i == 0 { None } else { Some(i - 1) });
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(None)
    }
}
