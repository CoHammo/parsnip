use icu_normalizer::ComposingNormalizer;
use std::{
    iter::{Skip, Take},
    str::CharIndices,
};

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
    pub source: &'c str,
    pub value: char,
    pub byte: usize,
}
impl<'c> Char<'c> {
    pub fn empty() -> Self {
        Self {
            source: "",
            value: '\0',
            byte: 0,
        }
    }

    pub fn new(source: &'c str, value: char, byte: usize) -> Self {
        Self {
            source,
            value,
            byte,
        }
    }

    pub fn owned(&self) -> Self {
        Self::new(self.source, self.value, self.byte)
    }

    pub fn next_byte(&self) -> usize {
        self.byte + self.value.len_utf8()
    }
}

pub struct ParseChars<'a> {
    value: &'a str,
    char_index: usize,
    chars: Take<Skip<CharIndices<'a>>>,
}
impl<'a> ParseChars<'a> {
    pub fn new(value: &'a str, from: Option<usize>, to: Option<usize>) -> Self {
        Self {
            value,
            char_index: 0,
            chars: value
                .char_indices()
                .skip(from.unwrap_or(0))
                .take(to.unwrap_or(value.len())),
        }
    }
}
impl<'a> Iterator for ParseChars<'a> {
    type Item = Char<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((byte, c)) = self.chars.next() {
            let ch = Some(Char::new(self.value, c, byte));
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
        Self {
            value: ComposingNormalizer::new_nfc()
                .normalize(&value)
                .into_owned(),
        }
    }

    pub fn chars(&self, from_char: Option<usize>, to_char: Option<usize>) -> ParseChars<'_> {
        ParseChars::new(&self.value, from_char, to_char)
    }
}
