mod class;
pub mod parser;
pub use parser::*;

use crate::{
    dispatch,
    parsers::Stat,
    types::{Tag, Token, Tokens},
};

use std::{any::Any, ops::RangeBounds};

// pub struct Base {
//     stat: Stat,
//     fresh: bool,
//     start: usize,
//     tokens: Tokens,
// }
// impl Base {
//     pub fn new() -> Self {
//         Self {
//             stat: Stat::Running,
//             fresh: true,
//             start: 0,
//             tokens: None,
//         }
//     }

//     pub fn reset(&mut self) {
//         self.stat = Stat::Running;
//         self.fresh = true;
//         self.start = 0;
//         self.tokens = None;
//     }

//     pub fn toks(&mut self) -> Tokens {
//         self.tokens.take()
//     }
// }

pub trait ParserDispatch<T: Matches> {
    // fn get_base_fn(&self) -> fn(me: &mut dyn Any) -> &mut Base;
    fn get_snip_fn(&self) -> fn(base: *mut ParserWrap<T>, me: &mut dyn Any, snip: &Snip<T>);
    fn get_finish_fn(&self) -> fn(base: *mut ParserWrap<T>, me: &mut dyn Any, snip: &Snip<T>);
    fn get_reset_fn(&self) -> fn(me: &mut dyn Any);
    fn get_string_fn(&self) -> fn(me: &dyn Any) -> String;
}

pub trait ParserT<T: Matches> {
    // fn base(&mut self) -> &mut Base;
    fn snip(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>);
    fn finish(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>);
    fn reset(&mut self);
    fn string(&self) -> String;
}

pub struct ParserWrap<T: Matches> {
    stat: Stat,
    fresh: bool,
    start: usize,
    tokens: Tokens,
    sub: Box<dyn Any>,
    // _base: fn(sub: &mut dyn Any) -> &mut Base,
    _snip: fn(base: *mut ParserWrap<T>, sub: &mut dyn Any, snip: &Snip<T>),
    _finish: fn(base: *mut ParserWrap<T>, sub: &mut dyn Any, snip: &Snip<T>),
    _reset: fn(sub: &mut dyn Any),
    _string: fn(sub: &dyn Any) -> String,
}
impl<T: Matches> ParserWrap<T> {
    pub fn make(sub: impl ParserDispatch<T> + 'static) -> Self {
        // let _base = sub.get_base_fn();
        let _snip = sub.get_snip_fn();
        let _finish = sub.get_finish_fn();
        let _reset = sub.get_reset_fn();
        let _string = sub.get_string_fn();
        Self {
            stat: Stat::Running,
            fresh: true,
            start: 0,
            tokens: None,
            sub: Box::new(sub),
            // _base,
            _snip,
            _finish,
            _reset,
            _string,
        }
    }

    pub fn add_tokens(&mut self, tokens: Tokens) {
        if let Some(new_tokens) = tokens {
            if let Some(toks) = &mut self.tokens {
                toks.extend(new_tokens);
            } else {
                self.tokens = Some(new_tokens);
            }
        }
    }

    pub fn parse(&mut self, source: impl ParseAs<T>, range: impl RangeBounds<usize>) -> Stat {
        let mut iter = source.snips(range);
        while let Some(snip) = iter.next() {
            (self._snip)(self, &mut *self.sub, &snip);
            match self.stat {
                Stat::Matched(_) | Stat::Failed => break,
                _ => {}
            }
        }
        if let Stat::Running = self.stat {
            (self._finish)(self, &mut *self.sub, &iter.item())
        }
        self.stat
    }

    // pub fn base(&mut self) -> &mut Base {
    //     (self._base)(&mut *self.sub)
    // }

    pub fn freshen(&mut self, index: usize) {
        if self.fresh {
            self.fresh = false;
            self.start = index;
        }
    }

    pub fn snip(&mut self, snip: &Snip<T>) -> Stat {
        (self._snip)(self, &mut *self.sub, snip);
        self.stat
    }

    pub fn finish(&mut self, snip: &Snip<T>) -> Stat {
        (self._finish)(self, &mut *self.sub, snip);
        self.stat
    }

    pub fn reset(&mut self) {
        if !self.fresh {
            self.stat = Stat::Running;
            self.fresh = true;
            self.start = 0;
            self.tokens = None;
            (self._reset)(&mut *self.sub)
        }
    }

    pub fn string(&self) -> String {
        (self._string)(&*self.sub)
    }
}

pub struct Its<T: Matches> {
    // pub base: Base,
    snips: Box<[T]>,
    len: usize,
    index: usize,
}

impl<T: Matches + 'static> Its<T> {
    pub fn new(value: impl ParseAs<T>) -> ParserWrap<T> {
        let snips = value.snip_store();
        let len = snips.len();
        let its = Self {
            // base: Base::new(),
            snips,
            len,
            index: 0,
        };
        ParserWrap::make(its)
    }
}

impl<T: Matches> ParserT<T> for Its<T> {
    // fn base(&mut self) -> &mut Base {
    //     &mut self.base
    // }

    fn snip(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>) {
        base.freshen(snip.index);
        if self.len == 0 {
            base.stat = Stat::Failed;
        } else if snip.value.matches(&self.snips[self.index]) {
            self.index += 1;
            if self.index == self.len {
                base.stat = Stat::Matched(snip.index + 1);
            }
        } else {
            base.stat = Stat::Failed;
        }
    }

    fn finish(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>) {
        if self.len == 0 {
            base.stat = Stat::Matched(snip.index + 1);
        } else {
            base.stat = Stat::Failed;
        };
    }

    fn reset(&mut self) {
        self.index = 0;
        // if !self.base.fresh {
        //     self.base.reset();
        //     self.index = 0;
        // }
    }

    fn string(&self) -> String {
        format!("Its({:?})", self.snips)
    }
}

dispatch!(Its);

pub struct Toks<T: Matches> {
    inner: ParserWrap<T>,
    tag: Tag,
}
impl<T: Matches + 'static> Toks<T> {
    pub fn new(parser: ParserWrap<T>, tag: Tag) -> ParserWrap<T> {
        let me = Self { inner: parser, tag };
        ParserWrap::make(me)
    }

    fn tokenize(&mut self, base: &mut ParserWrap<T>, end: usize) {
        base.tokens = Some(vec![Token::new(
            self.tag,
            base.start,
            end,
            base.tokens.take(),
        )]);
    }
}

impl<T: Matches + 'static> ParserT<T> for Toks<T> {
    fn snip(&mut self, base: &mut ParserWrap<T>, item: &Snip<T>) {
        match self.inner.snip(item) {
            Stat::Running => {}
            Stat::Matched(end) => {
                self.tokenize(base, end);
                base.stat = Stat::Matched(end);
            }
            Stat::Failed => base.stat = Stat::Failed,
        };
        base.fresh = self.inner.fresh;
    }

    fn finish(&mut self, base: &mut ParserWrap<T>, item: &Snip<T>) {
        let stat = self.inner.finish(item);
        match stat {
            Stat::Matched(end) => {
                self.tokenize(base, end);
                base.stat = Stat::Matched(end);
            }
            stat => base.stat = stat,
        };
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }
}

dispatch!(Toks);

pub struct Chains<T: Matches> {
    inners: Box<[ParserWrap<T>]>,
    len: usize,
    index: usize,
    check_at_index: usize,
}
impl<T: Matches + 'static> Chains<T> {
    pub fn new(parsers: Box<[ParserWrap<T>]>) -> ParserWrap<T> {
        let len = parsers.len();
        let me = Self {
            inners: parsers,
            len,
            index: 0,
            check_at_index: 0,
        };
        ParserWrap::make(me)
    }
}

impl<T: Matches + 'static> ParserT<T> for Chains<T> {
    fn snip(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>) {
        if snip.index >= self.check_at_index {
            base.freshen(snip.index);
            let parser = &mut self.inners[self.index];
            match parser.snip(snip) {
                Stat::Matched(end) => {
                    base.add_tokens(parser.tokens.take());
                    if self.index == self.len - 1 {
                        base.stat = Stat::Matched(end);
                    } else {
                        self.index += 1;
                        if end == snip.index {
                            self.snip(base, snip);
                        } else {
                            self.check_at_index = end;
                        }
                    }
                }
                Stat::Failed => base.stat = Stat::Failed,
                _ => {}
            }
        }
    }

    fn finish(&mut self, base: &mut ParserWrap<T>, snip: &Snip<T>) {
        let parser = &mut self.inners[self.index];
        match parser.finish(snip) {
            Stat::Matched(end) => {
                base.add_tokens(parser.tokens.take());
                if self.index == self.len - 1 {
                    base.stat = Stat::Matched(end);
                } else if end == snip.index {
                    self.index += 1;
                    self.finish(base, snip);
                } else {
                    base.stat = Stat::Failed;
                }
            }
            _ => base.stat = Stat::Failed,
        }
    }

    fn reset(&mut self) {
        for index in 0..=self.index {
            self.inners[index].reset();
        }
        self.index = 0;
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
}

dispatch!(Chains);
