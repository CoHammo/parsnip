use super::tags::Tag;

#[derive(Debug)]
pub struct Token {
    pub tag: Tag,
    pub start: usize,
    pub end: usize,
    pub len: usize,
    pub tokens: Tokens,
}
impl Token {
    pub fn new(tag: Tag, start: usize, end: usize, tokens: Tokens) -> Self {
        return Self {
            tag,
            start,
            end,
            len: end - start,
            tokens,
        };
    }
}

pub type Tokens = Option<Vec<Token>>;
