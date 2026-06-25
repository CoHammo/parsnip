use std::marker::PhantomData;

use super::Stat;
use super::types::*;

type Tokens = Option<Vec<Token>>;

pub trait ThingParser<T: PItem, I: Iterator<Item = T> + Clone>: Clone + std::fmt::Debug {
    fn take(&mut self, base: &mut GenParser<T, I>, item: &ParseItem<T, I>) -> Stat;
    fn finish(&mut self, base: &mut GenParser<T, I>, item: &ParseItem<T, I>) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}

#[derive(Debug)]
pub struct GenParser<T: PItem, I: Iterator<Item = T> + Clone> {
    pub stat: Stat,
    pub fresh: bool,
    pub start: usize,
    pub tokens: Tokens,
    pub sub: Box<dyn ThingParser<T, I>>,
}
// impl<T: PItem, P: ThingParser<T>> GenParser<T, P> {
//     pub fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
//         self.sub.take(self, item)
//     }
// }

/// Experimental Parsers
#[derive(Debug)]
pub struct Thing<T: PItem> {
    items: Box<[T]>,
    len: usize,
    index: usize,
}
impl<T: PItem> Thing<T> {
    pub fn new(value: impl Parses<T>) -> Self {
        let items = value.to_inner_store();
        let len = items.len();
        Self {
            items,
            len,
            index: 0,
        }
    }
}

impl<T: PItem> Clone for Thing<T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            len: self.len,
            index: 0,
        }
    }
}

impl<T: PItem> ThingParser<T> for Thing<T> {
    fn take<I: Iterator<Item = T> + Clone>(
        &mut self,
        base: &mut GenParser<T, Self>,
        item: &ParseItem<T, I>,
    ) -> Stat {
        if base.fresh {
            base.start = item.index();
            base.fresh = false;
        }
        if self.len == 0 {
            base.stat = Stat::Failed;
        } else if item.value.matches(&self.items[self.index]) {
            self.index += 1;
            if self.index == self.len {
                base.stat = Stat::Matched(item.index() + 1);
            }
        } else {
            base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}({}), byte={}, index={}, stat={:?}",
        //     self.bytes,
        //     self.bytes[self.index - 1],
        //     byte.value,
        //     byte.index(),
        //     self.base.stat
        // );
        base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(
        &mut self,
        base: &mut GenParser<T, Self>,
        item: &ParseItem<T, I>,
    ) -> Stat {
        if self.len == 0 {
            base.stat = Stat::Matched(item.index() + 1);
        } else {
            base.stat = Stat::Failed;
        };
        base.stat
    }

    fn reset(&mut self) {
        self.index = 0;
    }

    fn string(&self) -> String {
        format!("It({:?})", self.items)
    }
}

pub fn thing<T: PItem>(value: impl Parses<T>) -> Thing<T> {
    Thing::new(value)
}

/// Generic Tok
#[derive(Debug)]
pub struct Tokker<T: PItem, P: ThingParser<T>> {
    inner: GenParser<T, P>,
    tag: Tag,
}
impl<T: PItem, P: ThingParser<T>> Tokker<T, P> {
    pub fn new(parser: GenParser<T, P>, tag: Tag) -> Self {
        Self { inner: parser, tag }
    }
}

impl<T: PItem, P: ThingParser<T>> Tokker<T, P> {
    fn tokenize(&mut self, base: &mut GenParser<T, Self>, end: usize) {
        base.tokens = Some(vec![Token::new(
            self.tag,
            self.inner.start,
            end,
            self.inner.tokens.take(),
        )]);
    }
}
impl<T: PItem, P: ThingParser<T>> ThingParser<T> for Tokker<T, P> {
    fn take<I: Iterator<Item = T> + Clone>(
        &mut self,
        base: &mut GenParser<T, Self>,
        item: &ParseItem<T, I>,
    ) -> Stat {
        match self.inner.sub.take(&mut self.inner, item) {
            Stat::Running => {}
            Stat::Matched(end) => {
                self.tokenize(base, end);
                base.stat = Stat::Matched(end);
            }
            Stat::Failed => base.stat = Stat::Failed,
        };
        base.fresh = self.inner.fresh;
        base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(
        &mut self,
        base: &mut GenParser<T, Self>,
        item: &ParseItem<T, I>,
    ) -> Stat {
        match self.inner.finish(item) {
            Stat::Matched(end) => {
                self.tokenize(end);
                self.base.stat = Stat::Matched(end);
            }
            stat => self.base.stat = stat,
        };
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }
}

pub fn tokker<T: PItem, P: ThingParser<T>>(parser: GenParser<T, P>, tag: Tag) -> Tokker<T, P> {
    Tokker::new(parser, tag)
}
