use icu_normalizer::ComposingNormalizer;
use std::{
    ops::{Bound::*, RangeBounds},
    str::CharIndices,
};

#[derive(Debug, Clone)]
pub struct Char<'a> {
    pub value: char,
    pub byte: usize,
    chars: Option<&'a ParseChars<'a>>,
}
impl<'a> Char<'a> {
    pub fn new(value: char, byte: usize, chars: &'a ParseChars<'a>) -> Self {
        Self {
            value,
            byte,
            chars: Some(chars),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: '\0',
            byte: 0,
            chars: None,
        }
    }

    pub fn basic(value: char, byte: usize) -> Self {
        Self {
            value,
            byte,
            chars: None,
        }
    }

    pub fn with(&self, chars: &'a ParseChars<'a>) -> Self {
        Self {
            value: self.value,
            byte: self.byte,
            chars: Some(chars),
        }
    }

    pub fn peeks(&self) -> ParseChars<'a> {
        self.chars.unwrap().clone()
    }

    pub fn next_byte(&self) -> usize {
        self.byte + self.value.len_utf8()
    }
}

#[derive(Debug)]
pub struct ParseChars<'a> {
    fresh: bool,
    char_end: usize,
    char_index: usize,
    pub char: Char<'a>,
    chars: CharIndices<'a>,
}

impl Clone for ParseChars<'_> {
    fn clone(&self) -> Self {
        Self {
            fresh: true,
            char_end: self.char_end,
            char_index: self.char_index,
            char: self.char.clone(),
            chars: self.chars.clone(),
        }
    }
}

impl<'a> ParseChars<'a> {
    pub fn new(value: &'a str, range: impl RangeBounds<usize>) -> Self {
        let char_start = match range.start_bound() {
            Included(start) => *start,
            Excluded(start) => *start + 1,
            Unbounded => 0,
        };
        let char_end = match range.end_bound() {
            Included(end) => *end + 1,
            Excluded(end) => *end,
            Unbounded => value.len(),
        };
        let mut chars = value.char_indices();
        let mut i = 0;
        while i < char_start {
            chars.next();
            i += 1;
        }
        let c = if char_start < char_end
            && let Some((byte, c)) = chars.next()
        {
            Char::basic(c, byte)
        } else {
            Char::empty()
        };
        Self {
            fresh: true,
            char_end,
            char_index: char_start,
            char: c,
            chars,
        }
    }

    pub fn next<'b>(&'b mut self) -> Option<Char<'b>> {
        if self.char_index < self.char_end {
            if self.fresh {
                self.fresh = false;
                Some(self.char.with(self))
            } else if let Some((byte, c)) = self.chars.next() {
                self.char_index += 1;
                self.char = Char::basic(c, byte);
                Some(Char::new(c, byte, self))
            } else {
                None
            }
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

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn chars(&self, range: impl RangeBounds<usize>) -> ParseChars<'_> {
        ParseChars::new(&self.value, range)
    }
}
