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

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub opens: bool,
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
    StartLoop,
    CloseLoop(usize, usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Loop {
    pub start: usize,
    pub count: usize,
}
impl Loop {
    pub fn new(start: usize) -> Self {
        Self { start, count: 0 }
    }

    pub fn start(&mut self, start: usize) {
        self.start = start;
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.count = 0;
    }
}

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub loops: Vec<Loop>,
    // pub calls: Vec<usize>,
    pub prev_event: Option<usize>,
    pub prev_thread: Option<usize>,
    pub next_thread: Option<usize>,
}
impl Thread {
    pub fn new(ip: usize) -> Self {
        Self {
            ip,
            loops: Vec::new(),
            // calls: Vec::new(),
            prev_event: None,
            prev_thread: None,
            next_thread: None,
        }
    }
}

pub struct Threads {
    pool: Vec<Thread>,
    first: Option<usize>,
    index: Option<usize>,
    last: Option<usize>,
    next_free: Option<usize>,
}

impl Threads {
    pub fn new() -> Self {
        let mut me = Self {
            pool: vec![Thread::new(0), Thread::new(0)],
            first: Some(0),
            index: Some(0),
            last: Some(0),
            next_free: Some(1),
        };
        me.pool.push(Thread::new(0));
        me
    }

    fn at(&mut self, id: usize) -> &mut Thread {
        unsafe { self.pool.get_unchecked_mut(id) }
    }

    fn next(&mut self) -> Option<(usize, usize)> {
        if let Some(index) = self.index {
            let thread = self.at(index);
            let ip = thread.ip;
            self.index = thread.next_thread;
            Some((index, ip))
        } else {
            None
        }
    }

    fn free(&mut self, id: usize) {
        let thread = self.at(id);
        let prev_thread = thread.prev_thread;
        let next_thread = thread.next_thread;

        if let Some(prev) = prev_thread {
            self.at(prev).next_thread = next_thread;
        }
        if let Some(next) = next_thread {
            self.at(next).prev_thread = prev_thread;
        }
        if let Some(next_free) = self.next_free {
            self.at(next_free).next_thread = Some(id);
        } else {
            self.next_free = Some(id);
        }
        if let Some(first) = self.first
            && first == id
        {
            self.first = next_thread;
        }
        if let Some(last) = self.last
            && last == id
        {
            self.last = prev_thread;
        }
        self.at(id).prev_thread = None;
        self.at(id).next_thread = None;
    }

    fn fork(&mut self, id: usize, func: impl FnOnce(&mut Thread)) {
        let free_id = match self.next_free {
            Some(i) => {
                self.next_free = self.at(i).next_thread;
                i
            }
            None => {
                let i = self.pool.len();
                let t = Thread::new(0);
                self.pool.push(t);
                i
            }
        };
        let [thread, free] = unsafe { self.pool.get_disjoint_unchecked_mut([id, free_id]) };
        free.loops.clone_from(&thread.loops);
        free.prev_event = thread.prev_event;
        if let Some(last) = self.last {
            free.prev_thread = Some(last);
            func(free);
            self.at(last).next_thread = Some(free_id);
        } else {
            func(free);
        }
        self.last = Some(free_id);
    }

    fn restart(&mut self) -> bool {
        self.index = self.first;
        self.index.is_some()
    }
}

pub struct Parser<T: Matches> {
    stat: Stat,
    pub comms: Vec<Comm<T>>,
    pub threads: Threads,
    pub best_match: Option<Option<usize>>,
    pub events: Vec<Event>,
}

impl<T: Matches> Parser<T> {
    pub fn new(mut comms: Vec<Comm<T>>) -> Self {
        comms.push(Comm::Matched);
        Self {
            stat: Stat::Running,
            comms,
            threads: Threads::new(),
            best_match: None,
            events: Vec::with_capacity(8),
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
        while let Some((id, mut ip)) = self.threads.next() {
            loop {
                // println!(
                //     "Comm::{:?}, snipdex={}, snip={:?}",
                //     self.comms[ip], snip.index, snip.value
                // );
                match &self.comms[ip] {
                    Comm::Matched => {
                        self.best_match = Some(self.threads.at(id).prev_event);
                        // self.best_match = Some(thread.prev_event);
                        break;
                    }
                    Comm::Match(thing) => {
                        if !snip.end && thing.matches(&snip.value) {
                            // thread.ip += 1;
                            self.threads.at(id).ip = ip + 1;
                        } else {
                            self.threads.free(id);
                        }
                        break;
                    }
                    &Comm::Tok(opens) => {
                        let thread = self.threads.at(id);
                        let event = Event {
                            opens,
                            index: snip.index,
                            prev: thread.prev_event,
                        };
                        thread.prev_event = Some(self.events.len());
                        self.events.push(event);
                        ip += 1;
                    }
                    &Comm::StartLoop => {
                        let thread = self.threads.at(id);
                        thread.loops.push(Loop::new(ip + 1));
                        ip += 1;
                    }
                    &Comm::CloseLoop(min, max) => {
                        let thread = self.threads.at(id);
                        if let Some(loo) = thread.loops.last_mut() {
                            loo.count += 1;
                            if loo.count == max {
                                thread.loops.pop();
                                ip += 1;
                            } else {
                                ip = loo.start;
                                if loo.count >= min {
                                    self.threads.fork(id, |fork| {
                                        fork.ip = ip + 1;
                                        fork.loops.pop();
                                    });
                                }
                            }
                        } else {
                            println!("Tried to close a loop with no start");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                }
            } // Command Loop
        } // Threads Loop

        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
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
    let mut comms = vec![Comm::StartLoop];
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
