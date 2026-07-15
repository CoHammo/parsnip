mod tests;

use std::{
    ops::{
        Bound::{Excluded, Included, Unbounded},
        RangeBounds,
    },
    str::Bytes,
};

type Tokens = Option<Vec<Token>>;

#[derive(Debug, Clone)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub tokens: Tokens,
}
impl Token {
    pub fn new(start: usize) -> Self {
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

#[derive(Debug)]
pub struct Event {
    pub start: bool,
    pub index: usize,
    pub prev: Option<usize>,
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

pub trait ToComms<T: Matches> {
    fn to_comms(self) -> Vec<Comm<T>>;
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a str {
    fn snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
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
            index: self.index + 1,
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
    Match(T),
    Tok(bool),
    StartLoop(usize),
    CloseLoop(usize, usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Loop {
    pub start: usize,
    pub end: usize,
    pub count: usize,
    pub broken: bool,
}
impl Loop {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            count: 0,
            broken: false,
        }
    }

    pub fn start(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
        self.count = 0;
        self.broken = false;
    }
}

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub loops: Vec<Loop>,
    pub calls: Vec<usize>,
    pub event: Option<usize>,
}
impl Thread {
    pub fn new(ip: usize) -> Self {
        Self {
            ip,
            loops: Vec::new(),
            calls: Vec::new(),
            event: None,
        }
    }

    pub fn from_parent(ip: usize, parent: &Thread) -> Self {
        Self {
            ip,
            loops: parent.loops.clone(),
            calls: parent.calls.clone(),
            event: parent.event.clone(),
        }
    }

    pub fn start_from(&mut self, ip: usize, parent: &Thread) {
        self.ip = ip;
        self.loops.extend_from_slice(&parent.loops);
        self.calls.extend_from_slice(&parent.calls);
        self.event = parent.event;
    }

    pub fn kill(&mut self) {
        self.ip = 0;
        self.loops.clear();
        self.calls.clear();
        self.event = None;
    }
}

pub struct Parser<T: Matches> {
    stat: Stat,
    pub comms: Vec<Comm<T>>,
    pub threads: Vec<Thread>,
    pub free: Vec<usize>,
    pub active: Vec<usize>,
    pub next: Vec<usize>,
    pub matches: Vec<Option<usize>>,
    pub events: Vec<Event>,
    // toks: Vec<Token>,
    // pub tokens: Vec<Token>,
}

impl<T: Matches> Parser<T> {
    pub fn new(mut comms: Vec<Comm<T>>) -> Self {
        comms.push(Comm::Matched);
        Self {
            stat: Stat::Running,
            comms,
            threads: vec![Thread::new(0)],
            free: Vec::new(),
            active: vec![0],
            next: Vec::new(),
            matches: Vec::new(),
            events: Vec::new(),
            // toks: Vec::new(),
            // tokens: Vec::new(),
        }
    }

    pub fn add_thread(&mut self, parent: &Thread, ip: usize) {
        if let Some(tp) = self.free.pop() {
            self.threads[tp].start_from(ip, parent);
        } else {
            self.threads.push(Thread::from_parent(ip, parent));
        }
    }

    pub fn kill_thread(&mut self, thread_index: usize, thread: &mut Thread) {
        thread.kill();
        self.free.push(thread_index);
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
        let mut active_index = 0;
        while active_index < self.active.len() {
            let thread_index = self.active[active_index];
            let thread = &mut self.threads[thread_index];

            loop {
                // println!(
                //     "Comm::{:?}, snipdex={}, snip={:?}",
                //     self.comms[thread.ip], snip.index, snip.value
                // );
                match &self.comms[thread.ip] {
                    Comm::Matched => {
                        self.matches.push(thread.event);
                        thread.kill();
                        self.free.push(thread_index);
                        break;
                    }
                    Comm::Match(thing) => {
                        if !snip.end && thing.matches(&snip.value) {
                            thread.ip += 1;
                            self.next.push(thread_index);
                            break;
                        } else if let Some(loo) = thread.loops.last_mut() {
                            loo.broken = true;
                            thread.ip = loo.end;
                        } else {
                            thread.kill();
                            self.free.push(thread_index);
                            break;
                        }
                    }
                    &Comm::Tok(open) => {
                        let event = Event {
                            start: open,
                            index: snip.index,
                            prev: thread.event,
                        };
                        self.events.push(event);
                        thread.event = Some(self.events.len() - 1);
                        thread.ip += 1;
                    }
                    &Comm::StartLoop(len) => {
                        thread.loops.push(Loop::new(thread.ip + 1, len + 1));
                        thread.ip += 1;
                    }
                    &Comm::CloseLoop(min, max) => {
                        if let Some(loo) = thread.loops.last_mut() {
                            if loo.broken {
                                if loo.count >= min {
                                    thread.loops.pop();
                                    thread.ip += 1;
                                } else {
                                    thread.kill();
                                    self.free.push(thread_index);
                                    break;
                                }
                            } else {
                                loo.count += 1;
                                if loo.count == max {
                                    thread.loops.pop();
                                    thread.ip += 1;
                                } else {
                                    thread.ip = loo.start;
                                }
                            }
                        } else {
                            println!("Tried to close a loop with no start");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                }
            } // End of Command Loop

            active_index += 1;
        }

        self.active.clear();
        if self.next.is_empty() {
            if self.matches.is_empty() {
                self.stat = Stat::Failed;
            } else {
                self.stat = Stat::Matched;
            }
        } else {
            std::mem::swap(&mut self.next, &mut self.active);
        }
        self.stat
    }
}

impl<T: Matches> ToComms<T> for Vec<Comm<T>> {
    fn to_comms(self) -> Vec<Comm<T>> {
        self
    }
}

pub fn str<T: Matches>(value: impl ToComms<T>) -> Vec<Comm<T>> {
    value.to_comms()
}

pub fn tok<T: Matches>(value: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = value.to_comms();
    comms.insert(0, Comm::Tok(true));
    comms.push(Comm::Tok(false));
    comms
}

pub fn rep<T: Matches>(value: impl ToComms<T>, mut min: usize, mut max: usize) -> Vec<Comm<T>> {
    min = match min {
        0 => 1,
        m => m,
    };
    if max > 0 && max <= min {
        max = min;
    }
    let inner = value.to_comms();
    let len = inner.len();
    let mut comms = vec![Comm::StartLoop(len)];
    comms.extend(inner);
    comms.push(Comm::CloseLoop(min, max));
    comms
}

pub fn run<T: Matches>(values: Vec<impl ToComms<T>>) -> Vec<Comm<T>> {
    let mut all = Vec::new();
    for value in values {
        all.extend(value.to_comms());
    }
    all
}
