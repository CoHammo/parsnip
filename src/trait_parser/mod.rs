mod tests;

use super::types::{Stat, Tag};
use std::ops::{Bound::*, RangeBounds};
use std::str::Bytes;

pub trait Matches: Default + std::fmt::Debug + Clone {
    fn matches(&self, other: &Self) -> bool;
}

impl Matches for u8 {
    fn matches(&self, other: &u8) -> bool {
        self == other
    }
}

pub struct Snip<'a, T: Matches> {
    pub value: T,
    pub index: usize,
    iter: &'a dyn Snipper<T>,
}

impl<'a, T: Matches> Snip<'a, T> {
    pub fn peeks(&self) -> Box<dyn Snipper<T> + 'a> {
        self.iter.clone_box()
    }
}

pub trait ParseAs<T: Matches> {
    type Iter<'a>: Iterator<Item = T>
    where
        Self: 'a;
    fn snips(&self, range: impl RangeBounds<usize>) -> impl Snipper<T>;
    fn snip_store(self) -> Box<[T]>;
}

impl ParseAs<u8> for &str {
    type Iter<'a>
        = Bytes<'a>
    where
        Self: 'a;
    fn snips(&self, range: impl RangeBounds<usize>) -> impl Snipper<u8> {
        Snips::new(self.bytes(), self.len(), range)
    }

    fn snip_store(self) -> Box<[u8]> {
        self.as_bytes().into()
    }
}

pub trait Snipper<T: Matches> {
    fn item(&self) -> Snip<'_, T>;
    fn repeat(&mut self);
    fn next(&mut self) -> Option<Snip<'_, T>>;
    fn clone_box(&self) -> Box<dyn Snipper<T> + '_>;
}

pub struct Snips<T: Default, I: Iterator<Item = T>> {
    repeat: bool,
    index: usize,
    end: usize,
    item: T,
    iter: I,
}

impl<'a, T: Default, I: Iterator<Item = T>> Snips<T, I> {
    pub fn new(mut iter: I, source_len: usize, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Included(start) => *start,
            Excluded(start) => *start + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(end) => *end + 1,
            Excluded(end) => *end,
            Unbounded => source_len,
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

    fn _next(&mut self) -> bool {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                true
            } else if let Some(item) = self.iter.next() {
                self.index += 1;
                self.item = item;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Snipper<u8> for Snips<u8, Bytes<'_>> {
    fn item(&self) -> Snip<'_, u8> {
        Snip {
            value: self.item,
            index: self.index,
            iter: self,
        }
    }

    fn repeat(&mut self) {
        self.repeat = true;
    }

    fn next(&mut self) -> Option<Snip<'_, u8>> {
        if self._next() {
            Some(self.item())
        } else {
            None
        }
    }

    fn clone_box(&self) -> Box<dyn Snipper<u8> + '_> {
        Box::new(Self {
            repeat: true,
            index: self.index,
            end: self.end,
            item: self.item,
            iter: self.iter.clone(),
        })
    }
}

pub struct Parser<T: Matches> {
    pub events: Vec<Event>,
    pub inner: Box<dyn ParserT<T>>,
}
impl<T: Matches> Parser<T> {
    pub fn new(parser: impl ParserT<T> + 'static) -> Self {
        Self {
            events: Vec::with_capacity(16),
            inner: Box::new(parser),
        }
    }

    pub fn parse(&mut self, source: &impl ParseAs<T>) -> Stat {
        let mut iter = source.snips(..);
        while let Some(item) = iter.next() {
            match self.inner.snip(&mut self.events, &item) {
                Stat::Running => {}
                _ => break,
            }
        }
        match self.inner.base().stat {
            Stat::Running => self.inner.finish(&mut self.events, &iter.item()),
            _ => self.inner.base().stat,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub opens: bool,
    pub index: usize,
    pub prev: Option<usize>,
}

#[derive(Debug)]
pub struct Base {
    pub stat: Stat,
    pub fresh: bool,
    pub start: usize,
    pub prev_event: Option<usize>,
}
impl Base {
    pub fn new() -> Self {
        Self {
            stat: Stat::Running,
            fresh: true,
            start: 0,
            prev_event: None,
        }
    }

    pub fn reset(&mut self) {
        self.stat = Stat::Running;
        self.fresh = true;
        self.start = 0;
        self.prev_event = None;
    }

    // pub fn add_tokens(&mut self, tokens: Tokens) {
    //     if let Some(new_tokens) = tokens {
    //         if let Some(toks) = &mut self.events {
    //             toks.extend(new_tokens);
    //         } else {
    //             self.events = Some(new_tokens);
    //         }
    //     }
    // }
}

pub trait ParserT<T: Matches> {
    fn base(&mut self) -> &mut Base;
    fn snip(&mut self, toks: &mut Vec<Event>, item: &Snip<T>) -> Stat;
    fn finish(&mut self, toks: &mut Vec<Event>, item: &Snip<T>) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
    fn clone_box(&self) -> Box<dyn ParserT<T>>;
}

/// Experimental Parsers
#[derive(Debug)]
pub struct Str<T: Matches> {
    pub base: Base,
    items: Box<[T]>,
    len: usize,
    index: usize,
}
impl<T: Matches> Str<T> {
    pub fn new(value: impl ParseAs<T>) -> Self {
        let items = value.snip_store();
        let len = items.len();
        Self {
            base: Base::new(),
            items,
            len,
            index: 0,
        }
    }
}

impl<T: Matches + 'static> ParserT<T> for Str<T> {
    fn base(&mut self) -> &mut Base {
        &mut self.base
    }

    fn snip(&mut self, _: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        if self.base.fresh {
            self.base.start = item.index;
            self.base.fresh = false;
        }
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else if item.value.matches(&self.items[self.index]) {
            self.index += 1;
            if self.index == self.len {
                self.base.stat = Stat::Matched(item.index + 1);
            }
        } else {
            self.base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}, item={:?}, index={}, stat={:?}",
        //     self.items, item.value, item.index, self.base.stat
        // );
        self.base.stat
    }

    fn finish(&mut self, _: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        if self.len == 0 {
            self.base.stat = Stat::Matched(item.index + 1);
        } else {
            self.base.stat = Stat::Failed;
        };
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.index = 0;
    }

    fn string(&self) -> String {
        format!("Str({:?})", self.items)
    }

    fn clone_box(&self) -> Box<dyn ParserT<T>> {
        Box::new(self.clone())
    }
}

impl<T: Matches> Clone for Str<T> {
    fn clone(&self) -> Self {
        Self {
            base: Base::new(),
            items: self.items.clone(),
            len: self.len,
            index: 0,
        }
    }
}

pub fn str<T: Matches>(value: impl ParseAs<T>) -> Str<T> {
    Str::new(value)
}

/// Generic Tok
pub struct Tok<T: Matches> {
    pub base: Base,
    inner: Box<dyn ParserT<T>>,
    tag: Tag,
}
impl<T: Matches> Tok<T> {
    pub fn new(parser: impl ParserT<T> + 'static, tag: Tag) -> Self {
        Self {
            base: Base::new(),
            inner: Box::new(parser),
            tag,
        }
    }

    // fn tokenize(&mut self, end: usize) {
    //     self.base.tokens = Some(vec![Token::new(
    //         self.tag,
    //         self.base.start,
    //         end,
    //         self.base.tokens.take(),
    //     )]);
    // }
}

impl<T: Matches + 'static> ParserT<T> for Tok<T> {
    fn base(&mut self) -> &mut Base {
        &mut self.base
    }

    fn snip(&mut self, events: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        if self.base.fresh {
            self.base.start = item.index;
            self.base.prev_event = Some(events.len());
            events.push(Event {
                opens: true,
                index: item.index,
                prev: None,
            });
            self.base.fresh = false;
        }
        match self.inner.snip(events, item) {
            Stat::Running => {
                if let Some(index) = self.inner.base().prev_event {
                    let event = &mut events[index];
                    if event.prev.is_none() {
                        event.prev = self.base.prev_event;
                    }
                }
            }
            Stat::Matched(end) => {
                events.push(Event {
                    opens: false,
                    index: end,
                    prev: match self.inner.base().prev_event {
                        Some(index) => Some(index),
                        None => self.base.prev_event,
                    },
                });
                self.base.prev_event = Some(events.len() - 1);
                self.base.stat = Stat::Matched(end);
            }
            Stat::Failed => self.base.stat = Stat::Failed,
        };
        self.base.stat
    }

    fn finish(&mut self, events: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        match self.inner.finish(events, item) {
            Stat::Matched(end) => {
                events.push(Event {
                    opens: false,
                    index: end,
                    prev: match self.inner.base().prev_event {
                        Some(index) => Some(index),
                        None => self.base.prev_event,
                    },
                });
                self.base.prev_event = Some(events.len() - 1);
                self.base.stat = Stat::Matched(end);
            }
            _ => self.base.stat = Stat::Failed,
        }
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }

    fn clone_box(&self) -> Box<dyn ParserT<T>> {
        Box::new(Self {
            base: Base::new(),
            inner: self.inner.clone_box(),
            tag: self.tag,
        })
    }
}

pub fn tok<T: Matches>(parser: impl ParserT<T> + 'static, tag: Tag) -> Tok<T> {
    Tok::new(parser, tag)
}

pub struct Chain<T: Matches> {
    pub base: Base,
    inners: Box<[Box<dyn ParserT<T>>]>,
    len: usize,
    index: usize,
    check_at_index: usize,
}
impl<T: Matches> Chain<T> {
    pub fn new(parsers: Box<[Box<dyn ParserT<T>>]>) -> Self {
        let len = parsers.len();
        Self {
            base: Base::new(),
            inners: parsers,
            len,
            index: 0,
            check_at_index: 0,
        }
    }
}

// impl<T: Matches> Clone for Chain<T> {
//     fn clone(&self) -> Self {
//         Self {
//             base: Base::new(),
//             inners: self.inners.clone(),
//             len: self.len,
//             index: 0,
//             check_at_index: 0,
//         }
//     }
// }

impl<T: Matches + 'static> ParserT<T> for Chain<T> {
    fn base(&mut self) -> &mut Base {
        &mut self.base
    }

    fn snip(&mut self, events: &mut Vec<Event>, snip: &Snip<T>) -> Stat {
        if snip.index >= self.check_at_index {
            if self.base.fresh {
                self.base.start = snip.index;
                self.base.fresh = false;
            }
            let parser = &mut self.inners[self.index];
            match parser.snip(events, snip) {
                Stat::Running => {
                    if let Some(index) = parser.base().prev_event {
                        if let Some(prev) = self.base.prev_event
                            && prev != index
                            && let Some(event) = events.get_mut(index)
                            && event.prev.is_none()
                        {
                            event.prev = Some(prev);
                        }
                        self.base.prev_event = Some(index);
                    }
                }
                Stat::Matched(end) => {
                    self.index += 1;
                    self.base.prev_event = parser.base().prev_event;
                    if self.index == self.len {
                        self.base.stat = Stat::Matched(end);
                    } else {
                        if end == snip.index {
                            self.snip(events, snip);
                        } else {
                            self.check_at_index = end;
                        }
                    }
                }
                Stat::Failed => self.base.stat = Stat::Failed,
            }
        }
        self.base.stat
    }

    fn finish(&mut self, events: &mut Vec<Event>, snip: &Snip<T>) -> Stat {
        let parser = &mut self.inners[self.index];
        match parser.finish(events, snip) {
            Stat::Matched(end) => {
                self.index += 1;
                self.base.prev_event = parser.base().prev_event;
                if self.index == self.len {
                    self.base.stat = Stat::Matched(end);
                } else if end == snip.index {
                    self.finish(events, snip);
                } else {
                    self.base.stat = Stat::Failed;
                }
            }
            _ => self.base.stat = Stat::Failed,
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            for parser in self.inners.iter_mut() {
                parser.reset();
            }
            self.index = 0;
        }
    }

    fn string(&self) -> String {
        format!(
            "Run([{}])",
            self.inners
                .iter()
                .map(|p| p.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }

    fn clone_box(&self) -> Box<dyn ParserT<T>> {
        let inners = self
            .inners
            .iter()
            .map(|p| p.clone_box())
            .collect::<Vec<Box<dyn ParserT<T>>>>();
        Box::new(Self {
            base: Base::new(),
            inners: inners.into_boxed_slice(),
            len: self.len,
            index: 0,
            check_at_index: 0,
        })
    }
}

pub fn chain<T: Matches>(parsers: &[&dyn ParserT<T>]) -> Chain<T> {
    let inners = parsers
        .iter()
        .map(|p| p.clone_box())
        .collect::<Vec<Box<dyn ParserT<T>>>>()
        .into_boxed_slice();
    Chain::new(inners)
}

pub struct Rep<T: Matches> {
    pub base: Base,
    inner: Box<dyn ParserT<T>>,
    min: usize,
    max: usize,
    count: usize,
    end: usize,
    first_event: Option<usize>,
    last_event: Option<usize>,
}
impl<T: Matches> Rep<T> {
    pub fn new(parser: impl ParserT<T> + 'static, mut min: usize, mut max: usize) -> Self {
        min = match min {
            0 => 1,
            m => m,
        };
        if max > 0 && max <= min {
            max = min;
        }
        Self {
            base: Base::new(),
            inner: Box::new(parser),
            min,
            max,
            count: 0,
            end: 0,
            first_event: None,
            last_event: None,
        }
    }
}

impl<T: Matches + 'static> ParserT<T> for Rep<T> {
    fn base(&mut self) -> &mut Base {
        &mut self.base
    }

    fn snip(&mut self, events: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        if self.base.fresh {
            self.base.start = item.index;
            self.base.fresh = false;
            let mut iter = item.peeks();
            while let Some(snip) = iter.next() {
                match self.inner.snip(events, &snip) {
                    Stat::Running => {
                        if let Some(index) = self.inner.base().prev_event {
                            if let Some(prev) = self.base.prev_event
                                && prev != index
                                && let Some(event) = events.get_mut(index)
                                && event.prev.is_none()
                            {
                                event.prev = Some(prev);
                            }
                            self.base.prev_event = Some(index)
                        }
                    }
                    Stat::Matched(end) => {
                        self.count += 1;
                        self.end = end;
                        self.base.prev_event = self.inner.base().prev_event;
                        self.inner.reset();
                        if self.count == self.max {
                            self.base.stat = Stat::Matched(end);
                            break;
                        } else if end == item.index {
                            iter.repeat();
                        }
                    }
                    Stat::Failed => {
                        if self.count >= self.min {
                            self.base.stat = Stat::Matched(self.end);
                        } else {
                            self.base.stat = Stat::Failed;
                        }
                        break;
                    }
                }
            }
            self.base.prev_event = self.first_event;
        } else {
            self.base.prev_event = self.last_event;
        }
        self.base.stat
    }

    fn finish(&mut self, events: &mut Vec<Event>, item: &Snip<T>) -> Stat {
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.count = 0;
            self.base.reset();
            self.inner.reset();
        }
    }

    fn string(&self) -> String {
        format!("Rep({:?})", self.inner.string())
    }

    fn clone_box(&self) -> Box<dyn ParserT<T>> {
        Box::new(Self {
            base: Base::new(),
            inner: self.inner.clone_box(),
            min: self.min,
            max: self.max,
            count: 0,
            end: 0,
            first_event: None,
            last_event: None,
        })
    }
}

pub fn rep<T: Matches>(parser: impl ParserT<T> + 'static, min: usize, max: usize) -> Rep<T> {
    Rep::new(parser, min, max)
}
