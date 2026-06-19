use super::tags::Tag;

#[derive(Debug)]
pub struct Token {
    pub tag: Tag,
    pub start_byte: usize,
    pub end_byte: usize,
    pub len: usize,
    pub tokens: Option<Vec<Token>>,
}
impl Token {
    pub fn new(tag: Tag, start_byte: usize, end_byte: usize, tokens: Option<Vec<Token>>) -> Self {
        return Self {
            tag,
            start_byte,
            end_byte,
            len: end_byte - start_byte,
            tokens,
        };
    }
}
