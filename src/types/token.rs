use super::tags::Tag;

#[derive(Debug)]
pub struct Token {
    pub tag: Tag,
    pub start: usize,
    pub end: usize,
    pub len: usize,
    pub tokens: Option<Vec<Token>>,
}
impl Token {
    pub fn new(tag: Tag, start: usize, end: usize, tokens: Option<Vec<Token>>) -> Self {
        return Self {
            tag,
            start,
            end,
            len: end - start,
            tokens,
        };
    }
}
