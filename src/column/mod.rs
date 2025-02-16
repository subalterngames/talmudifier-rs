use crate::word::Word;

mod cosmic;
mod width;

pub trait ColumnMaker {
    fn get_num_lines(&mut self, words: &[Word]) -> usize;

    fn get_words(&mut self, words: &[Word], num_lines: usize) -> Option<usize> {
        if num_lines == 0 {
            return None;
        }
        for i in 0..words.len() {
            // We exceeded the number of lines. Break!
            if self.get_num_lines(&words[..i]) > num_lines {
                return if i == 0 { None } else { Some(i - 1) };
            }
        }
        None
    }
}
