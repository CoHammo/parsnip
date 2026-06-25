use std::{ops::RangeBounds, str::Bytes};

use super::*;
use icu_normalizer::ComposingNormalizer;

// #[derive(Debug, Clone)]
// pub struct Char<'a> {
//     pub value: char,
//     pub byte: usize,
//     chars: Option<&'a ParseChars<'a>>,
// }
// impl<'a> Char<'a> {
//     pub fn new(value: char, byte: usize, chars: &'a ParseChars<'a>) -> Self {
//         Self {
//             value,
//             byte,
//             chars: Some(chars),
//         }
//     }

//     pub fn empty() -> Self {
//         Self {
//             value: '\0',
//             byte: 0,
//             chars: None,
//         }
//     }

//     pub fn basic(value: char, byte: usize) -> Self {
//         Self {
//             value,
//             byte,
//             chars: None,
//         }
//     }

//     pub fn with(&self, chars: &'a ParseChars<'a>) -> Self {
//         Self {
//             value: self.value,
//             byte: self.byte,
//             chars: Some(chars),
//         }
//     }

//     pub fn peeks(&self) -> ParseChars<'a> {
//         self.chars.unwrap().clone()
//     }

//     pub fn next_byte(&self) -> usize {
//         self.byte + self.value.len_utf8()
//     }
// }

// #[derive(Debug)]
// pub struct ParseChars<'a> {
//     repeat: bool,
//     index: usize,
//     end: usize,
//     char: Char<'a>,
//     chars: CharIndices<'a>,
// }

// impl Clone for ParseChars<'_> {
//     fn clone(&self) -> Self {
//         Self {
//             repeat: true,
//             index: self.index,
//             end: self.end,
//             char: self.char.clone(),
//             chars: self.chars.clone(),
//         }
//     }
// }

// impl<'a> ParseChars<'a> {
//     pub fn new(value: &'a str, range: impl RangeBounds<usize>) -> Self {
//         let start = match range.start_bound() {
//             Included(start) => *start,
//             Excluded(start) => *start + 1,
//             Unbounded => 0,
//         };
//         let end = match range.end_bound() {
//             Included(end) => *end + 1,
//             Excluded(end) => *end,
//             Unbounded => value.len(),
//         };
//         let mut chars = value.char_indices();
//         for _ in 0..start {
//             chars.next();
//         }
//         let c = if start < end
//             && let Some((byte, c)) = chars.next()
//         {
//             Char::basic(c, byte)
//         } else {
//             Char::empty()
//         };
//         Self {
//             repeat: true,
//             end,
//             index: start,
//             char: c,
//             chars,
//         }
//     }

//     pub fn repeat(&mut self) {
//         self.repeat = true;
//     }

//     pub fn char(&self) -> Char<'_> {
//         self.char.with(self)
//     }

//     pub fn next(&mut self) -> Option<Char<'_>> {
//         if self.index < self.end {
//             if self.repeat {
//                 self.repeat = false;
//                 Some(self.char.with(self))
//             } else if let Some((byte, c)) = self.chars.next() {
//                 self.index += 1;
//                 self.char = Char::basic(c, byte);
//                 Some(self.char.with(self))
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }

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
}

impl Parses<u8> for Text {
    type Iter<'a>
        = Bytes<'a>
    where
        Self: 'a;

    fn to_parse_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<u8, Bytes<'_>> {
        ParseIter::new(self.value.bytes(), self.value.len(), range)
    }

    fn to_inner_store(&self) -> Box<[u8]> {
        Box::new([])
    }
}

// impl ToParseIter<u8> for Text {
//     fn to_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<'_, u8> {
//         ParseIter::new(self.value.as_bytes().iter(), self.value.len(), range)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Byte<'a> {
//     pub value: u8,
//     iter: Option<&'a BytesIter<'a>>,
// }
// impl<'a> Byte<'a> {
//     pub fn new(value: u8, iter: &'a BytesIter<'a>) -> Self {
//         Self {
//             value,
//             iter: Some(iter),
//         }
//     }

//     pub fn index(&self) -> usize {
//         self.iter.unwrap().index
//     }
// }

// #[derive(Debug)]
// pub struct BytesIter<'a> {
//     repeat: bool,
//     index: usize,
//     end: usize,
//     byte: u8,
//     iter: Bytes<'a>,
// }

// impl Clone for BytesIter<'_> {
//     fn clone(&self) -> Self {
//         Self {
//             repeat: true,
//             index: self.index,
//             end: self.end,
//             byte: self.byte,
//             iter: self.iter.clone(),
//         }
//     }
// }

// impl<'a> BytesIter<'a> {
//     pub fn new(value: &'a str, range: impl RangeBounds<usize>) -> Self {
//         let start = match range.start_bound() {
//             Included(start) => *start,
//             Excluded(start) => *start + 1,
//             Unbounded => 0,
//         };
//         let end = match range.end_bound() {
//             Included(end) => *end + 1,
//             Excluded(end) => *end,
//             Unbounded => value.len(),
//         };
//         let byte: u8;
//         let iter = if start < end {
//             let mut bytes = value.bytes();
//             for _ in 0..start {
//                 bytes.next();
//             }
//             byte = match bytes.next() {
//                 Some(b) => b,
//                 None => 0,
//             };
//             bytes
//         } else {
//             byte = 0;
//             "".bytes()
//         };
//         Self {
//             repeat: true,
//             index: start,
//             end,
//             byte,
//             iter,
//         }
//     }

//     pub fn byte(&self) -> Byte<'_> {
//         Byte::new(self.byte, self)
//     }

//     pub fn repeat(&mut self) {
//         self.repeat = true;
//     }

//     pub fn next(&mut self) -> Option<Byte<'_>> {
//         if self.index < self.end {
//             if self.repeat {
//                 self.repeat = false;
//                 Some(Byte::new(self.byte, self))
//             } else if let Some(b) = self.iter.next() {
//                 self.index += 1;
//                 self.byte = b;
//                 Some(Byte::new(self.byte, self))
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }

// pub trait PItem: Default + Clone + PartialEq + std::fmt::Debug {
//     fn from_iter(&self) -> Self;
// }
// impl PItem for u8 {
//     fn from_iter(&self) -> u8 {
//         *self
//     }
// }

// pub trait BoxArray<T: PItem> {
//     fn box_array(&self) -> Box<[T]>;
// }

// impl BoxArray<u8> for &str {
//     fn box_array(&self) -> Box<[u8]> {
//         let norm = ComposingNormalizer::new_nfc().normalize(self);
//         norm.as_bytes().into()
//     }
// }

// #[derive(Debug, Clone)]
// pub struct ParseItem<'a, T: PItem> {
//     pub value: T,
//     iter: &'a ParseIter<'a, T>,
// }
// impl<'a, T: PItem> ParseItem<'a, T> {
//     pub fn new(value: T, iter: &'a ParseIter<'a, T>) -> Self {
//         Self { value, iter }
//     }

//     pub fn peeks(&self) -> ParseIter<'_, T> {
//         self.iter.clone()
//     }

//     pub fn index(&self) -> usize {
//         self.iter.index
//     }
// }

// #[derive(Debug)]
// pub struct ParseIter<'a, T: PItem> {
//     repeat: bool,
//     index: usize,
//     end: usize,
//     item: T,
//     items: Iter<'a, T>,
// }

// impl<T: PItem> Clone for ParseIter<'_, T> {
//     fn clone(&self) -> Self {
//         Self {
//             repeat: true,
//             index: self.index,
//             end: self.end,
//             item: self.item.clone(),
//             items: self.items.clone(),
//         }
//     }
// }

// pub trait ToParseIter<T: PItem> {
//     fn to_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<'_, T>;
// }

// impl<'a, T: PItem> ParseIter<'a, T> {
//     pub fn new(mut iter: Iter<'a, T>, len: usize, range: impl RangeBounds<usize>) -> Self {
//         let start = match range.start_bound() {
//             Included(start) => *start,
//             Excluded(start) => *start + 1,
//             Unbounded => 0,
//         };
//         let end = match range.end_bound() {
//             Included(end) => *end + 1,
//             Excluded(end) => *end,
//             Unbounded => len,
//         };
//         let item: T;
//         let items = if start < end {
//             for _ in 0..start {
//                 iter.next();
//             }
//             item = match iter.next() {
//                 Some(i) => i.from_iter(),
//                 None => T::default(),
//             };
//             iter
//         } else {
//             item = T::default();
//             [].iter()
//         };
//         Self {
//             repeat: true,
//             index: start,
//             end,
//             item,
//             items,
//         }
//     }

//     pub fn item(&self) -> ParseItem<'_, T> {
//         ParseItem::new(self.item.from_iter(), self)
//     }

//     pub fn repeat(&mut self) {
//         self.repeat = true;
//     }

//     pub fn next(&mut self) -> Option<ParseItem<'_, T>> {
//         if self.index < self.end {
//             if self.repeat {
//                 self.repeat = false;
//                 Some(ParseItem::new(self.item.from_iter(), self))
//             } else if let Some(item) = self.items.next() {
//                 self.index += 1;
//                 self.item = item.from_iter();
//                 Some(ParseItem::new(self.item.from_iter(), self))
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }
