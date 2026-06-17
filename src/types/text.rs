use icu_normalizer::ComposingNormalizer;
use std::{
    collections::VecDeque,
    iter::{Skip, Take},
    ops::{Bound::*, RangeBounds},
    str::CharIndices,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ParseChars<'a> {
    value: &'a str,
    char_index: usize,
    pub char: Char<'a>,
    char_buffer: VecDeque<Char<'a>>,
    chars: Take<Skip<CharIndices<'a>>>,
}
impl<'a> ParseChars<'a> {
    pub fn new(value: &'a str, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Included(start) => *start,
            Excluded(start) => *start + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(end) => *end + 1,
            Excluded(end) => *end,
            Unbounded => value.len(),
        };
        Self {
            value,
            char_index: 0,
            char: Char::empty(),
            char_buffer: VecDeque::new(),
            chars: value.char_indices().skip(start).take(end),
        }
    }
}
// impl<'a> Iterator for ParseChars<'a> {
//     type Item = Char<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(c) = self.char_buffer.pop_front() {
//             self.current_char = c.clone();
//             Some(c)
//         } else if let Some((byte, c)) = self.chars.next() {
//             self.current_char = Char::new(self.value, c, byte);
//             self.char_index += 1;
//             Some(self.current_char.clone())
//         } else {
//             None
//         }
//     }
// }
impl<'a> ParseChars<'a> {
    pub fn next(&mut self) -> bool {
        if let Some(c) = self.char_buffer.pop_front() {
            self.char = c;
            self.char_index += 1;
            true
        } else if let Some((byte, c)) = self.chars.next() {
            self.char = Char::new(self.value, c, byte);
            self.char_index += 1;
            true
        } else {
            false
        }
    }

    pub fn peek(&mut self, offset: usize) -> Option<&Char<'a>> {
        if offset == 0 {
            Some(&self.char)
        } else {
            while offset - 1 >= self.char_buffer.len()
                && let Some((byte, c)) = self.chars.next()
            {
                let ch = Char::new(self.value, c, byte);
                self.char_buffer.push_back(ch);
            }
            if let ch @ Some(_) = self.char_buffer.get(offset - 1) {
                ch
            } else {
                None
            }
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
