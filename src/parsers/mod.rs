mod alt;
mod dbg;
mod not;
mod rec;
mod rep;
mod run;
mod str;
mod till;
mod tok;

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
    pub start_byte: usize,
    pub fresh: bool,
    pub tokens: Option<Vec<Token>>,
}
impl BaseParser {
    pub fn new() -> Self {
        Self {
            stat: Stat::Running,
            start_byte: 0,
            fresh: true,
            tokens: None,
        }
    }

    pub fn add_tokens(&mut self, tokens: Option<Vec<Token>>) {
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
        self.start_byte = 0;
        self.fresh = true;
        self.tokens = None;
    }
}

pub trait CharParser {
    fn take_char(&mut self, ch: &Char) -> Stat;
    fn finish(&mut self, ch: &Char) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}

parser_enum!(Dbg, Str, Tok, Not, Run, Rep, Till, Alt);
