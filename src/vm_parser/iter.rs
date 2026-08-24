use super::*;
use std::{
    ops::{Bound, RangeBounds},
    str::Bytes,
};

pub trait Matches: Default + std::fmt::Debug + Clone {
    fn matches(&self, other: &Self) -> bool;
    fn copy_snip(&self) -> Self;
}

impl Matches for u8 {
    fn matches(&self, other: &u8) -> bool {
        self == other
    }

    fn copy_snip(&self) -> u8 {
        *self
    }
}

#[derive(Debug)]
pub struct Snip<'a, T: Matches, I: SnipIter<T>> {
    pub value: Option<T>,
    pub index: usize,
    snips: &'a Snips<T, I>,
}

impl<'a, T: Matches, I: SnipIter<T>> Snip<'a, T, I> {
    pub fn peek_snips(&self) -> Snips<T, I> {
        self.snips.clone()
    }
}

pub trait AsSnips<T: Matches, I: SnipIter<T>> {
    fn as_snips(&self, range: impl RangeBounds<usize>) -> Snips<T, I>;
}

pub trait ToComms<T: Matches> {
    fn to_comms(self) -> Vec<Comm<T>>;
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a str {
    fn as_snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
        Snips::new(self.bytes(), self.len(), range)
    }
}

impl ToComms<u8> for &str {
    fn to_comms(self) -> Vec<Comm<u8>> {
        self.bytes().map(|b| Comm::Match(b)).collect()
    }
}

pub trait SnipIter<T: Matches>: Iterator<Item = T> + Clone {}
impl<T: Matches, I: Iterator<Item = T> + Clone> SnipIter<T> for I {}

#[derive(Debug)]
pub struct Snips<T: Matches, I: SnipIter<T>> {
    repeat: bool,
    index: usize,
    end: usize,
    terminated: bool,
    snip: T,
    iter: I,
}

impl<T: Matches, I: SnipIter<T>> Snips<T, I> {
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
        let snip = match iter.next() {
            Some(s) => s,
            None => T::default(),
        };
        Self {
            repeat: true,
            index: start,
            end,
            terminated: false,
            snip,
            iter,
        }
    }

    fn get_snip(&self) -> Snip<'_, T, I> {
        Snip {
            value: Some(self.snip.copy_snip()),
            index: self.index,
            snips: self,
        }
    }

    pub fn next(&mut self) -> Option<Snip<'_, T, I>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.get_snip())
            } else if let Some(snip) = self.iter.next() {
                self.index += 1;
                self.snip = snip;
                Some(self.get_snip())
            } else {
                self.index += 1;
                self.end = self.index;
                self.terminated = true;
                Some(Snip {
                    value: None,
                    index: self.index,
                    snips: self,
                })
            }
        } else {
            if !self.terminated {
                Some(Snip {
                    value: None,
                    index: self.index,
                    snips: self,
                })
            } else {
                None
            }
        }
    }
}

impl<T: Matches, I: SnipIter<T>> Clone for Snips<T, I> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            terminated: self.terminated,
            snip: self.snip.copy_snip(),
            iter: self.iter.clone(),
        }
    }
}
