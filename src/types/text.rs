use icu_normalizer::ComposingNormalizer;
use std::{
    ops::{Bound::*, RangeBounds},
    slice::Iter,
    str::{Bytes, CharIndices},
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
    repeat: bool,
    index: usize,
    end: usize,
    char: Char<'a>,
    chars: CharIndices<'a>,
}

impl Clone for ParseChars<'_> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            char: self.char.clone(),
            chars: self.chars.clone(),
        }
    }
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
        let mut chars = value.char_indices();
        for _ in 0..start {
            chars.next();
        }
        let c = if start < end
            && let Some((byte, c)) = chars.next()
        {
            Char::basic(c, byte)
        } else {
            Char::empty()
        };
        Self {
            repeat: true,
            end,
            index: start,
            char: c,
            chars,
        }
    }

    pub fn repeat(&mut self) {
        self.repeat = true;
    }

    pub fn char(&self) -> Char<'_> {
        self.char.with(self)
    }

    pub fn next(&mut self) -> Option<Char<'_>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.char.with(self))
            } else if let Some((byte, c)) = self.chars.next() {
                self.index += 1;
                self.char = Char::basic(c, byte);
                Some(self.char.with(self))
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

    // pub fn chars(&self, range: impl RangeBounds<usize>) -> ParseChars<'_> {
    //     ParseChars::new(&self.value, range)
    // }

    pub fn bytes(&self, range: impl RangeBounds<usize>) -> BytesIter<'_> {
        BytesIter::new(&self.value, range)
    }
}

impl ToParseIter<u8> for Text {
    fn to_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<'_, u8> {
        ParseIter::new(self.value.as_bytes().iter(), self.value.len(), range)
    }
}

#[derive(Debug, Clone)]
pub struct Byte<'a> {
    pub value: u8,
    // pub index: usize,
    iter: Option<&'a BytesIter<'a>>,
}
impl<'a> Byte<'a> {
    pub fn new(value: u8) -> Self {
        Self {
            value,
            // index,
            iter: None,
        }
    }

    pub fn with(&self, iter: &'a BytesIter<'a>) -> Self {
        Self {
            value: self.value,
            // index: self.index,
            iter: Some(iter),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: 0,
            // index: 0,
            iter: None,
        }
    }

    pub fn index(&self) -> usize {
        self.iter.unwrap().index
    }
}

#[derive(Debug)]
pub struct BytesIter<'a> {
    repeat: bool,
    index: usize,
    end: usize,
    byte: Byte<'a>,
    iter: Bytes<'a>,
}

impl Clone for BytesIter<'_> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            byte: self.byte.clone(),
            iter: self.iter.clone(),
        }
    }
}

impl<'a> BytesIter<'a> {
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
        let byte: Byte;
        let iter = if start < end {
            let mut bytes = value.bytes();
            for _ in 0..start {
                bytes.next();
            }
            byte = match bytes.next() {
                Some(i) => Byte::new(i),
                None => Byte::empty(),
            };
            bytes
        } else {
            byte = Byte::empty();
            "".bytes()
        };
        Self {
            repeat: true,
            index: start,
            end,
            byte,
            iter,
        }
    }

    pub fn byte(&self) -> Byte<'_> {
        self.byte.with(self)
    }

    pub fn repeat(&mut self) {
        self.repeat = true;
    }

    pub fn next(&mut self) -> Option<Byte<'_>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.byte.with(self))
            } else if let Some(b) = self.iter.next() {
                self.index += 1;
                self.byte = Byte::new(b);
                Some(self.byte.with(self))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait PI: Default + Clone + PartialEq + std::fmt::Debug {}
impl<T: Default + Clone + PartialEq + std::fmt::Debug> PI for T {}

#[derive(Debug, Clone)]
pub struct ParseItem<T: PI>(pub T);
impl<T: PI> ParseItem<T> {
    pub fn empty() -> Self {
        Self(T::default())
    }

    pub fn with<'a>(&self, items: &'a ParseIter<'a, T>) -> IterItem<'a, T> {
        IterItem::new(self.0.clone(), items)
    }
}

pub trait ToParseItems<T: PI> {
    fn to_items(&self) -> Box<[ParseItem<T>]>;
}

impl ToParseItems<u8> for &str {
    fn to_items(&self) -> Box<[ParseItem<u8>]> {
        let norm = ComposingNormalizer::new_nfc().normalize(self);
        let bytes = norm.as_bytes();
        bytes.iter().map(|b| ParseItem(*b)).collect::<Box<_>>()
    }
}

#[derive(Debug, Clone)]
pub struct IterItem<'a, T: PI> {
    pub value: T,
    // pub index: usize,
    iter: &'a ParseIter<'a, T>,
}
impl<'a, T: PI> IterItem<'a, T> {
    pub fn new(value: T, iter: &'a ParseIter<'a, T>) -> Self {
        Self {
            value,
            // index,
            iter,
        }
    }

    pub fn peeks(&self) -> ParseIter<'_, T> {
        self.iter.clone()
    }

    pub fn index(&self) -> usize {
        self.iter.index
    }
}

#[derive(Debug)]
pub struct ParseIter<'a, T: PI> {
    repeat: bool,
    index: usize,
    end: usize,
    item: ParseItem<T>,
    items: Iter<'a, T>,
}

impl<T: PI> Clone for ParseIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            item: self.item.clone(),
            items: self.items.clone(),
        }
    }
}

pub trait ToParseIter<T: PI> {
    fn to_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<'_, T>;
}

impl<'a, T: PI> ParseIter<'a, T> {
    pub fn new(mut iter: Iter<'a, T>, len: usize, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Included(start) => *start,
            Excluded(start) => *start + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(end) => *end + 1,
            Excluded(end) => *end,
            Unbounded => len,
        };
        let item: ParseItem<T>;
        let items = if start < end {
            for _ in 0..start {
                iter.next();
            }
            item = match iter.next() {
                Some(i) => ParseItem(i.clone()),
                None => ParseItem::empty(),
            };
            iter
        } else {
            item = ParseItem::empty();
            [].iter()
        };
        Self {
            repeat: true,
            index: start,
            end,
            item,
            items,
        }
    }

    pub fn item(&self) -> IterItem<'_, T> {
        self.item.with(self)
    }

    pub fn repeat(&mut self) {
        self.repeat = true;
    }

    pub fn next(&mut self) -> Option<IterItem<'_, T>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.item.with(self))
            } else if let Some(b) = self.items.next() {
                self.index += 1;
                self.item = ParseItem(b.clone());
                Some(self.item.with(self))
            } else {
                None
            }
        } else {
            None
        }
    }
}
