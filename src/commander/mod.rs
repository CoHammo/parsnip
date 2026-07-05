use std::{
    ops::{
        Bound::{Excluded, Included, Unbounded},
        RangeBounds,
    },
    str::Bytes,
};

type Tokens = Option<Vec<Token>>;

#[derive(Debug)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub tokens: Tokens,
}
impl Token {
    pub fn start(start: usize) -> Self {
        Self {
            start,
            end: 0,
            tokens: None,
        }
    }

    pub fn close(&mut self, end: usize) {
        self.end = end;
    }

    pub fn add(&mut self, tok: Token) {
        if let Some(toks) = &mut self.tokens {
            toks.push(tok);
        } else {
            self.tokens = Some(vec![tok]);
        }
    }
}

pub trait Matches: Default + std::fmt::Debug + Clone {
    fn matches(&self, other: &Self) -> bool;
    fn snip_copy(&self) -> Self;
}

impl Matches for u8 {
    fn matches(&self, other: &u8) -> bool {
        self == other
    }

    fn snip_copy(&self) -> u8 {
        *self
    }
}

#[derive(Debug)]
pub struct Snip<'a, T: Matches, I: SnipIter<T>> {
    pub value: T,
    pub index: usize,
    pub end: bool,
    snips: &'a Snips<T, I>,
}

impl<'a, T: Matches, I: SnipIter<T>> Snip<'a, T, I> {
    pub fn peeks(&self) -> Snips<T, I> {
        self.snips.clone()
    }
}

pub trait AsSnips<T: Matches, I: SnipIter<T>> {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<T, I>;
}

pub trait ToChecks<T: Matches> {
    fn to_checks(self) -> Vec<Comm<T>>;
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a str {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
        Snips::new(self.bytes(), self.len(), range)
    }
}

impl ToChecks<u8> for &str {
    fn to_checks(self) -> Vec<Comm<u8>> {
        self.bytes().map(|b| Comm::Check(b)).collect()
    }
}

pub trait SnipIter<T: Matches>: Iterator<Item = T> + Clone {}
impl<T: Matches, I: Iterator<Item = T> + Clone> SnipIter<T> for I {}

#[derive(Debug)]
pub struct Snips<T: Matches, I: SnipIter<T>> {
    repeat: bool,
    index: usize,
    end: usize,
    snip: T,
    iter: I,
}

impl<T: Matches, I: SnipIter<T>> Snips<T, I> {
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
        let snip: T = if start < end {
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
            snip,
            iter,
        }
    }

    pub fn snip(&self) -> Snip<'_, T, I> {
        Snip {
            value: self.snip.snip_copy(),
            index: self.index,
            end: false,
            snips: self,
        }
    }

    pub fn end_snip(&self) -> Snip<'_, T, I> {
        Snip {
            value: self.snip.snip_copy(),
            index: self.index,
            end: true,
            snips: self,
        }
    }

    pub fn next(&mut self) -> Option<Snip<'_, T, I>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.snip())
            } else if let Some(snip) = self.iter.next() {
                self.index += 1;
                self.snip = snip;
                Some(self.snip())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T: Matches, I: SnipIter<T>> Clone for Snips<T, I> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            snip: self.snip.snip_copy(),
            iter: self.iter.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    Running,
    Matched,
    Failed,
}

#[derive(Debug, Clone)]
pub enum Comm<T: Matches> {
    Matched,
    Check(T),
    MultiCheck(Box<[(T, usize)]>),
    Tok(bool),
    StartLoop(usize),
    EndLoop(usize, usize),
}

#[derive(Debug)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
    pub count: usize,
    pub broken: bool,
}

pub struct CommandParser<T: Matches> {
    stat: Stat,
    comms: Vec<Comm<T>>,
    comm: usize,
    loops: Vec<Loop>,
    utoks: Vec<Token>,
    pub tokens: Vec<Token>,
}

impl<T: Matches> CommandParser<T> {
    pub fn new(mut comms: Vec<Comm<T>>) -> Self {
        comms.push(Comm::Matched);
        Self {
            stat: Stat::Running,
            comms,
            comm: 0,
            loops: Vec::new(),
            utoks: Vec::new(),
            tokens: Vec::new(),
        }
    }

    pub fn parse<I: SnipIter<T>>(&mut self, source: impl AsSnips<T, I>) -> Stat {
        let mut iter = source.snips(..);
        while let Some(snip) = iter.next() {
            match self.take_snip(&snip) {
                Stat::Running => {}
                _ => break,
            }
        }
        self.take_snip(&iter.end_snip());
        self.stat
    }

    pub fn take_snip<I: SnipIter<T>>(&mut self, snip: &Snip<T, I>) -> Stat {
        // println!("Next Snip...");
        loop {
            // println!(
            //     "Comm::{:?}, snipdex={}, snip={:?}",
            //     self.comms[self.comm], snip.index, snip.value
            // );
            let comm: &Comm<T>;
            unsafe {
                comm = self.comms.get_unchecked(self.comm);
            }
            match comm {
                Comm::Matched => {
                    self.stat = Stat::Matched;
                    break;
                }
                Comm::Check(check) => {
                    if !snip.end && check.matches(&snip.value) {
                        self.comm += 1;
                        break;
                    } else if let Some(loo) = self.loops.last_mut() {
                        self.comm = loo.end;
                        loo.broken = true;
                    } else {
                        self.stat = Stat::Failed;
                    }
                }
                Comm::MultiCheck(checks) => {
                    let mut matched = false;
                    for (check, next) in checks.iter() {
                        if check.matches(&snip.value) {
                            matched = true;
                            self.comm += next;
                            break;
                        }
                    }
                    if !matched {
                        self.stat = Stat::Failed;
                    }
                    break;
                }
                &Comm::Tok(open) => {
                    if open {
                        self.utoks.push(Token::start(snip.index));
                    } else {
                        if let Some(mut tok) = self.utoks.pop() {
                            tok.end = snip.index + 1;
                            if let Some(utok) = self.utoks.last_mut() {
                                utok.add(tok);
                            } else {
                                self.tokens.push(tok);
                            }
                        } else {
                            println!("Tried to close a token with no opening");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    self.comm += 1;
                }
                &Comm::StartLoop(len) => {
                    self.loops.push(Loop {
                        start: self.comm + 1,
                        end: self.comm + len + 1,
                        count: 0,
                        broken: false,
                    });
                    self.comm += 1;
                }
                &Comm::EndLoop(min, max) => {
                    if let Some(loo) = self.loops.last_mut() {
                        if loo.broken {
                            if loo.count >= min {
                                self.loops.pop();
                                self.comm += 1;
                            } else {
                                self.stat = Stat::Failed;
                                break;
                            }
                        } else {
                            loo.count += 1;
                            if loo.count == max {
                                self.loops.pop();
                                self.comm += 1;
                            } else {
                                self.comm = loo.start;
                            }
                        }
                    } else {
                        println!("Tried to end a loop while not in a loop");
                        self.stat = Stat::Failed;
                        break;
                    }
                }
            }
        }
        self.stat
    }
}

impl<T: Matches> ToChecks<T> for Vec<Comm<T>> {
    fn to_checks(self) -> Vec<Comm<T>> {
        self
    }
}

pub fn str<T: Matches>(value: impl ToChecks<T>) -> Vec<Comm<T>> {
    value.to_checks()
}

pub fn tok<T: Matches>(value: impl ToChecks<T>) -> Vec<Comm<T>> {
    let mut comms = value.to_checks();
    comms.insert(0, Comm::Tok(true));
    comms.push(Comm::Tok(false));
    comms
}

pub fn rep<T: Matches>(value: impl ToChecks<T>, mut min: usize, mut max: usize) -> Vec<Comm<T>> {
    min = match min {
        0 => 1,
        m => m,
    };
    if max > 0 && max <= min {
        max = min;
    }
    let mut comms = value.to_checks();
    let len = comms.len();
    comms.insert(0, Comm::StartLoop(len));
    comms.push(Comm::EndLoop(min, max));
    comms
}

pub fn run<T: Matches>(values: Vec<impl ToChecks<T>>) -> Vec<Comm<T>> {
    let mut all = Vec::new();
    for value in values {
        all.extend(value.to_checks());
    }
    all
}
