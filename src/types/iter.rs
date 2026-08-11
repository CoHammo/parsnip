use super::*;
use std::ops::{Bound::*, RangeBounds};

pub struct Item<S, V: PartialEq> {
    source: S,
    pub value: V,
    pub index: usize,
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
                Some(self.item())
            } else if let Some(item) = self.iter.next() {
                self.index += 1;
                self.item = item.from_iter();
                Some(self.item())
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
