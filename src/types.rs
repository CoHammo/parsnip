use icu_normalizer::ComposingNormalizer;
use std::{
    collections::HashMap,
    iter::{Skip, Take},
    str::CharIndices,
};

#[derive(Debug, Clone, Copy)]
pub enum Stat {
    Running,
    PossibleMatch(usize),
    Matched(usize),
    Failed,
}

pub struct Tags {
    count: u32,
    names: HashMap<Tag, String>,
    ids: HashMap<String, Tag>,
}
impl Tags {
    pub fn new() -> Self {
        let mut names = HashMap::new();
        names.insert(Tag(0), "Text".into());
        let mut ids = HashMap::new();
        ids.insert("Text".into(), Tag(0));
        Self {
            count: 1,
            names,
            ids,
        }
    }

    pub fn none(&self) -> Tag {
        Tag(0)
    }

    pub fn add(&mut self, name: &str) {
        self.names.insert(Tag(self.count), name.into());
        self.ids.insert(name.into(), Tag(self.count));
        self.count += 1;
    }

    pub fn tag(&mut self, name: &str) -> Tag {
        if let Some(tag) = self.ids.get(name) {
            *tag
        } else {
            self.add(name);
            self.tag(name)
        }
    }

    pub fn get(&self, tag: Tag) -> &str {
        if let Some(name) = self.names.get(&tag) {
            name
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Tag(u32);

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

#[derive(Debug, Clone, Copy)]
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
