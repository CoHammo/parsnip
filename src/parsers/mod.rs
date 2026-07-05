mod alt;
mod dbg;
mod not;
mod rec;
mod rep;
mod run;
mod str;
mod till;
mod tok;
pub mod trait_parsers;

use super::*;
pub use alt::*;
pub use dbg::*;
pub use not::*;
// pub use rec::*;
pub use rep::*;
pub use run::*;
use std::ops::RangeBounds;
pub use str::*;
pub use till::*;
pub use tok::*;
pub use trait_parsers::*;

#[derive(Debug, Clone, Copy)]
pub enum Stat {
    Running,
    // PossibleMatch(usize),
    Matched(usize),
    Failed,
}
#[derive(Debug)]
pub struct BaseParser {
    pub stat: Stat,
    pub start: usize,
    pub fresh: bool,
    pub tokens: Tokens,
}
impl BaseParser {
    pub fn new() -> Self {
        Self {
            stat: Stat::Running,
            start: 0,
            fresh: true,
            tokens: None,
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

    pub fn reset(&mut self) {
        self.stat = Stat::Running;
        self.start = 0;
        self.fresh = true;
        self.tokens = None;
    }
}

impl Clone for BaseParser {
    fn clone(&self) -> Self {
        Self::new()
    }
}

pub trait ItemParser<T: PItem>: Clone {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat;
    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}

parser_enum!(Dbg, It, Tok, Not, Run, Rep, Till, Alt);
