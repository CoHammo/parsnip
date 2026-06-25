use super::super::*;
use std::ops::{Bound::*, RangeBounds};

#[derive(Debug)]
pub struct Rep<T: PItem> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
    min: usize,
    max: usize,
    count: usize,
    end: usize,
}
impl<T: PItem> Rep<T> {
    pub fn new(parser: Parser<T>, range: impl RangeBounds<usize>) -> Self {
        let min = match range.start_bound() {
            Included(&m) | Excluded(&m) => match m {
                0 => 1,
                _ => m,
            },
            Unbounded => 1,
        };
        let max = match range.end_bound() {
            Included(&m) => {
                if m < min {
                    min
                } else {
                    m
                }
            }
            Excluded(&m) => {
                if m < min {
                    min
                } else {
                    m - 1
                }
            }
            Unbounded => 0,
        };

        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            min,
            max,
            count: 0,
            end: 0,
        }
    }
}
pub fn rep<T: PItem>(parser: Parser<T>, range: impl RangeBounds<usize>) -> Parser<T> {
    Parser::Rep(Rep::new(parser, range))
}

impl<T: PItem> Clone for Rep<T> {
    fn clone(&self) -> Self {
        Rep::new(*self.inner.clone(), self.min..=self.max)
    }
}

impl<T: PItem> Rep<T> {
    fn look<I: Iterator<Item = T> + Clone>(&mut self, ch: &ParseItem<T, I>) {
        let mut peeks = ch.peeks();
        while let Some(it) = peeks.next() {
            match self.inner.take(&it) {
                Stat::Matched(end) => {
                    self.count += 1;
                    self.end = end;
                    self.base.add_tokens(self.inner.take_tokens());
                    self.inner.reset();
                    if self.count == self.max {
                        self.base.stat = Stat::Matched(end);
                        break;
                    } else if end == it.index() {
                        peeks.repeat();
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
                _ => {}
            }
        }
    }
}

impl<T: PItem> ItemParser<T> for Rep<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        freshen!(self, item, {
            self.look(item);
        });

        // match self.inner.take_char(ch) {
        //     Stat::Matched(end_byte) => {
        //         self.count += 1;
        //         self.end_byte = end_byte;
        //         self.base.add_tokens(self.inner.take_tokens());
        //         self.inner.reset();
        //         if self.count == self.mmax {
        //             self.base.stat = Stat::Matched(end_byte);
        //         } else if self.count >= self.mmin {
        //             self.base.stat = Stat::PossibleMatch(end_byte);
        //         }
        //     }
        //     Stat::Failed => {
        //         if self.count >= self.mmin {
        //             self.base.stat = Stat::Matched(self.end_byte);
        //         } else {
        //             self.base.stat = Stat::Failed;
        //         }
        //     }
        //     stat => self.base.stat = stat,
        // }
        self.base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        if !self.inner.fresh() {
            match self.inner.finish(item) {
                Stat::Matched(end) => {
                    self.count += 1;
                    self.end = end;
                    self.base.stat = Stat::Matched(end);
                }
                _ => {}
            }
        }
        if self.count >= self.min {
            self.base.add_tokens(self.inner.take_tokens());
            self.inner.reset();
            self.base.stat = Stat::Matched(self.end);
        } else {
            self.base.stat = Stat::Failed;
        };
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            self.inner.reset();
            self.count = 0;
            self.end = 0;
        }
    }

    fn string(&self) -> String {
        format!("Rep({})", self.inner.string())
    }
}
