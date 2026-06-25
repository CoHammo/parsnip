use std::{
    ops::{Bound::*, RangeBounds},
    str::{Bytes, CharIndices},
};

pub trait PItem: Default + Clone + std::fmt::Debug {
    fn from_iter(&self) -> Self;
    fn matches(&self, other: &Self) -> bool;
}

pub trait Parses<T: PItem> {
    type Iter<'a>: Iterator<Item = T> + Clone
    where
        Self: 'a;
    fn to_parse_iter<'a>(&'a self, range: impl RangeBounds<usize>) -> ParseIter<T, Self::Iter<'a>>;
    fn to_inner_store(&self) -> Box<[T]>;
}

impl PItem for u8 {
    fn from_iter(&self) -> u8 {
        *self
    }
    fn matches(&self, other: &u8) -> bool {
        self == other
    }
}

impl Parses<u8> for &str {
    type Iter<'a>
        = Bytes<'a>
    where
        Self: 'a;
    fn to_parse_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<u8, Bytes<'_>> {
        ParseIter::new(self.bytes(), self.len(), range)
    }

    fn to_inner_store(&self) -> Box<[u8]> {
        self.as_bytes().into()
    }
}

impl PItem for (usize, char) {
    fn from_iter(&self) -> (usize, char) {
        *self
    }

    fn matches(&self, other: &(usize, char)) -> bool {
        self.1 == other.1
    }
}

impl Parses<(usize, char)> for &str {
    type Iter<'a>
        = CharIndices<'a>
    where
        Self: 'a;
    fn to_parse_iter(
        &self,
        range: impl RangeBounds<usize>,
    ) -> ParseIter<(usize, char), CharIndices<'_>> {
        ParseIter::new(self.char_indices(), self.len(), range)
    }

    fn to_inner_store(&self) -> Box<[(usize, char)]> {
        self.char_indices().collect()
    }
}

#[derive(Debug)]
pub struct ParseItem<'a, T: PItem, I: Iterator<Item = T> + Clone> {
    pub value: T,
    parse_iter: &'a ParseIter<T, I>,
}

impl<'a, T: PItem, I: Iterator<Item = T> + Clone> ParseItem<'a, T, I> {
    pub fn index(&self) -> usize {
        self.parse_iter.index
    }

    pub fn peeks(&self) -> ParseIter<T, I> {
        self.parse_iter.clone()
    }
}

#[derive(Debug)]
pub struct ParseIter<T: PItem, I: Iterator<Item = T> + Clone> {
    repeat: bool,
    index: usize,
    end: usize,
    item: T,
    iter: I,
}

impl<'a, T: PItem, I: Iterator<Item = T> + Clone + 'a> ParseIter<T, I> {
    pub fn new(mut iter: I, len: usize, range: impl RangeBounds<usize>) -> Self {
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
        let item: T = if start < end {
            for _ in 0..start {
                iter.next();
            }
            match iter.next() {
                Some(i) => i,
                None => T::default(),
            }
        } else {
            T::default()
        };
        Self {
            repeat: true,
            index: start,
            end,
            item,
            iter,
        }
    }

    pub fn item(&self) -> ParseItem<'_, T, I> {
        ParseItem {
            value: self.item.from_iter(),
            parse_iter: self,
        }
    }

    pub fn repeat(&mut self) {
        self.repeat = true;
    }

    pub fn next(&mut self) -> Option<ParseItem<'_, T, I>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(ParseItem {
                    value: self.item.from_iter(),
                    parse_iter: self,
                })
            } else if let Some(item) = self.iter.next() {
                self.index += 1;
                self.item = item.from_iter();
                Some(ParseItem {
                    value: item.from_iter(),
                    parse_iter: self,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, T: PItem, I: Iterator<Item = T> + Clone + 'a> Clone for ParseIter<T, I> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            item: self.item.clone(),
            iter: self.iter.clone(),
        }
    }
}
