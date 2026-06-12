use icu_normalizer::ComposingNormalizer;
use std::{
    collections::HashMap,
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
pub struct BaseParser {
    pub stat: Stat,
    pub start_byte: usize,
    pub fresh: bool,
    pub tokens: Option<Vec<Token>>,
}
impl BaseParser {
    pub fn new() -> Self {
        Self {
            stat: Stat::Running,
            start_byte: 0,
            fresh: true,
            tokens: None,
        }
    }

    pub fn add_tokens(&mut self, tokens: Option<Vec<Token>>) {
        if let Some(new_tokens) = tokens {
            if let Some(toks) = &mut self.tokens {
                toks.extend(new_tokens);
            } else {
                self.tokens = Some(new_tokens);
            }
        }
    }

    pub fn reset(&mut self) {
        self.stat = Stat::Running;
        self.start_byte = 0;
        self.fresh = true;
        self.tokens = None;
    }
}

pub struct Tagger {
    count: u32,
    to_tag: HashMap<Tag, String>,
    to_id: HashMap<String, Tag>,
}
impl Tagger {
    pub fn new() -> Self {
        let mut to_tag = HashMap::new();
        to_tag.insert(Tag(0), "".to_string());
        let mut to_id = HashMap::new();
        to_id.insert("".to_string(), Tag(0));
        Self {
            count: 1,
            to_tag,
            to_id,
        }
    }

    pub fn none(&self) -> Tag {
        Tag(0)
    }

    pub fn add(&mut self, name: &str) {
        self.to_tag.insert(Tag(self.count), name.to_string());
        self.to_id.insert(name.to_string(), Tag(self.count));
        self.count += 1;
    }

    pub fn at(&mut self, name: &str) -> Tag {
        if let Some(tag) = self.to_id.get(name) {
            *tag
        } else {
            self.add(name);
            self.at(name)
        }
    }

    pub fn get(&self, tag: Tag) -> &str {
        self.to_tag.get(&tag).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Tag(u32);

#[derive(Debug)]
pub struct Token {
    pub tag: Tag,
    pub start_byte: usize,
    pub end_byte: usize,
    pub tokens: Option<Vec<Token>>,
}
impl Token {
    pub fn new(tag: Tag, start_byte: usize, end_byte: usize, tokens: Option<Vec<Token>>) -> Self {
        return Self {
            tag,
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
