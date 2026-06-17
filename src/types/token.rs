use super::tags::Tag;

#[derive(Debug)]
pub struct Token {
    pub tag: Tag,
    // pub value: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub tokens: Option<Vec<Token>>,
}
impl Token {
    pub fn new(
        tag: Tag,
        // source: Option<&str>,
        start_byte: usize,
        end_byte: usize,
        tokens: Option<Vec<Token>>,
    ) -> Self {
        return Self {
            tag,
            // value: source.map(|s| s[start_byte..end_byte].to_string()),
            start_byte,
            end_byte,
            tokens,
        };
    }
}
