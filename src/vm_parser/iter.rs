use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
    str::Bytes,
};

pub trait Parses: Default + std::fmt::Debug + Clone {
    // fn bytes_len() -> u8;
    fn matches(&self, other: &[u8]) -> bool;
    fn to_bytes(self) -> Vec<u8>;
}

pub struct Snip<T: Parses> {
    pub value: Option<T>,
    pub index: usize,
}

impl<T: Parses> Snip<T> {
    pub fn new(value: T, index: usize) -> Self {
        Self {
            value: Some(value),
            index,
        }
    }

    pub fn empty(index: usize) -> Self {
        Self { value: None, index }
    }
}

pub trait SnipIter<T: Parses>: Iterator<Item = T> {}
impl<T: Parses, I: Iterator<Item = T>> SnipIter<T> for I {}

pub trait AsSnips<T: Parses, I: SnipIter<T>> {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<T, I>;
}

#[derive(Debug)]
pub struct Snips<T: Parses, I: SnipIter<T>> {
    index: usize,
    end: usize,
    terminated: bool,
    iter: I,
    data: PhantomData<T>,
}

impl<T: Parses, I: SnipIter<T>> Snips<T, I> {
    pub fn new(mut iter: I, source_len: usize, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };
        let mut end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => source_len,
        };
        if end > source_len {
            end = source_len;
        }
        for _ in 0..start {
            iter.next();
        }
        Self {
            index: start,
            end,
            terminated: false,
            iter,
            data: PhantomData,
        }
    }

    pub fn next(&mut self) -> Option<Snip<T>> {
        if self.index < self.end {
            if let Some(val) = self.iter.next() {
                let snip = Snip::new(val, self.index);
                self.index += 1;
                return Some(snip);
            } else {
                self.end = self.index;
            }
        }
        if !self.terminated {
            self.terminated = true;
            return Some(Snip::empty(self.index));
        } else {
            None
        }
    }
}

impl Parses for u8 {
    // fn bytes_len() -> u8 {
    //     1
    // }

    fn matches(&self, other: &[u8]) -> bool {
        self == unsafe { other.get_unchecked(0) }
    }

    fn to_bytes(self) -> Vec<u8> {
        vec![self]
    }
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a str {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
        Snips::new(self.bytes(), self.len(), range)
    }
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a String {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
        Snips::new(self.bytes(), self.len(), range)
    }
}
