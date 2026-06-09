use std::iter::{Skip, Take};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

#[derive(Debug, Clone, Copy)]
pub enum Stat {
    Running,
    HasMatch(usize),
    Matched(usize),
    Failed,
}

#[derive(Debug)]
pub struct Token {
    pub kind: Option<String>,
    pub value: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub tokens: Option<Vec<Token>>,
}
impl Token {
    pub fn new(
        kind: Option<String>,
        value: Option<&str>,
        start_byte: usize,
        end_byte: usize,
        tokens: Option<Vec<Token>>,
    ) -> Self {
        return Self {
            kind,
            value: value.map(|v| v.to_string()),
            start_byte,
            end_byte,
            tokens,
        };
    }
}

#[derive(Debug)]
pub struct Char<'c> {
    pub full_string: &'c str,
    pub value: &'c str,
    pub byte_offset: usize,
}
impl<'c> Char<'c> {
    pub fn empty() -> Self {
        Self {
            full_string: "",
            value: "",
            byte_offset: 0,
        }
    }

    pub fn new(full_string: &'c str, value: &'c str, byte_offset: usize) -> Self {
        Self {
            full_string,
            value,
            byte_offset,
        }
    }

    pub fn renew(&self) -> Self {
        Self::new(self.full_string, self.value, self.byte_offset)
    }

    pub fn next_byte_offset(&self) -> usize {
        self.byte_offset + self.value.len()
    }
}

pub struct ParseChars<'a> {
    value: &'a str,
    char_index: usize,
    graphemes: Take<Skip<GraphemeIndices<'a>>>,
}
impl<'a> ParseChars<'a> {
    pub fn new(value: &'a str) -> Self {
        Self {
            value,
            char_index: 0,
            graphemes: value.grapheme_indices(true).skip(0).take(value.len()),
        }
    }

    pub fn range(value: &'a str, from_char: usize, to_char: Option<usize>) -> Self {
        Self {
            value,
            char_index: from_char,
            graphemes: value
                .grapheme_indices(true)
                .skip(from_char)
                .take(to_char.unwrap_or(value.len())),
        }
    }
}
impl<'a> Iterator for ParseChars<'a> {
    type Item = Char<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((byte_offset, ch)) = self.graphemes.next() {
            let ch = Some(Char::new(self.value, ch, byte_offset));
            self.char_index += 1;
            ch
        } else {
            None
        }
    }
}

pub struct Text {
    value: String,
}
impl Text {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn chars(&self) -> ParseChars<'_> {
        ParseChars::new(&self.value)
    }

    pub fn chars_range(&self, from_char: usize, to_char: Option<usize>) -> ParseChars<'_> {
        ParseChars::range(&self.value, from_char, to_char)
    }
}
