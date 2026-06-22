use super::super::*;
use std::ops::{Bound::*, RangeBounds};

#[derive(Debug)]
pub struct Rep {
    pub base: BaseParser,
    inner: Box<Parser>,
    min: usize,
    max: usize,
    count: usize,
    end_byte: usize,
}
impl Rep {
    pub fn new(parser: Parser, range: impl RangeBounds<usize>) -> Self {
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
            end_byte: 0,
        }
    }
}
pub fn rep(parser: Parser, range: impl RangeBounds<usize>) -> Parser {
    Parser::Rep(Rep::new(parser, range))
}

impl Clone for Rep {
    fn clone(&self) -> Self {
        Rep::new(*self.inner.clone(), self.min..=self.max)
    }
}

impl Rep {
    fn look(&mut self, ch: &Char) {
        let mut peeks = ch.peeks();
        while let Some(c) = peeks.next() {
            match self.inner.take_char(&c) {
                Stat::Matched(byte) => {
                    self.count += 1;
                    self.end_byte = byte;
                    self.base.add_tokens(self.inner.take_tokens());
                    self.inner.reset();
                    if byte == c.byte {
                        peeks.repeat();
                    }
                    if self.count == self.max {
                        self.base.stat = Stat::Matched(byte);
                        break;
                    }
                }
                Stat::Failed => {
                    if self.count >= self.min {
                        self.base.stat = Stat::Matched(self.end_byte);
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

impl CharParser for Rep {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch, {
            self.look(ch);
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

    fn finish(&mut self, ch: &Char) -> Stat {
        if !self.inner.fresh() {
            match self.inner.finish(ch) {
                Stat::Matched(end_byte) => {
                    self.count += 1;
                    self.end_byte = end_byte;
                    self.base.stat = Stat::Matched(end_byte);
                }
                _ => {}
            }
        }
        if self.count >= self.min {
            self.base.add_tokens(self.inner.take_tokens());
            self.inner.reset();
            self.base.stat = Stat::Matched(self.end_byte);
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
            self.end_byte = 0;
        }
    }

    fn string(&self) -> String {
        format!("Rep({})", self.inner.string())
    }
}
