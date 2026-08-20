mod tests;

use std::{
    ops::{
        Bound::{self},
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
    pub start: bool,
    pub index: usize,
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

pub trait Matches: Default + std::fmt::Debug + Clone {
    fn matches(&self, other: &Self) -> bool;
    fn copy_snip(&self) -> Self;
}

impl Matches for u8 {
    fn matches(&self, other: &u8) -> bool {
        self == other
    }

    fn copy_snip(&self) -> u8 {
        *self
    }
}

#[derive(Debug)]
pub struct Snip<'a, T: Matches, I: SnipIter<T>> {
    pub value: Option<T>,
    pub index: usize,
    snips: &'a Snips<T, I>,
}

impl<'a, T: Matches, I: SnipIter<T>> Snip<'a, T, I> {
    pub fn peek_snips(&self) -> Snips<T, I> {
        self.snips.clone()
    }
}

pub trait AsSnips<T: Matches, I: SnipIter<T>> {
    fn as_snips(&self, range: impl RangeBounds<usize>) -> Snips<T, I>;
}

pub trait ToComms<T: Matches> {
    fn to_comms(self) -> Vec<Comm<T>>;
}

impl<'a> AsSnips<u8, Bytes<'a>> for &'a str {
    fn as_snips(&self, range: impl RangeBounds<usize>) -> Snips<u8, Bytes<'a>> {
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
    terminated: bool,
    snip: T,
    iter: I,
}

impl<T: Matches, I: SnipIter<T>> Snips<T, I> {
    pub fn new(mut iter: I, source_len: usize, range: impl RangeBounds<usize>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(s) => *s,
            Bound::Excluded(s) => *s + 1,
            Bound::Unbounded => 0,
        };
        let mut end = match range.end_bound() {
            Bound::Included(e) => *e + 1,
            Bound::Excluded(e) => *e,
            Bound::Unbounded => source_len,
        };
        if end > source_len {
            end = source_len;
        }
        for _ in 0..start {
            iter.next();
        }
        let snip = match iter.next() {
            Some(s) => s,
            None => T::default(),
        };
        Self {
            repeat: true,
            index: start,
            end,
            terminated: false,
            snip,
            iter,
        }
    }

    fn get_snip(&self) -> Snip<'_, T, I> {
        Snip {
            value: Some(self.snip.copy_snip()),
            index: self.index,
            snips: self,
        }
    }

    pub fn next(&mut self) -> Option<Snip<'_, T, I>> {
        if self.index < self.end {
            if self.repeat {
                self.repeat = false;
                Some(self.get_snip())
            } else if let Some(snip) = self.iter.next() {
                self.index += 1;
                self.snip = snip;
                Some(self.get_snip())
            } else {
                self.index += 1;
                self.end = self.index;
                self.terminated = true;
                Some(Snip {
                    value: None,
                    index: self.index,
                    snips: self,
                })
            }
        } else {
            if !self.terminated {
                Some(Snip {
                    value: None,
                    index: self.index,
                    snips: self,
                })
            } else {
                None
            }
        }
    }
}

impl<T: Matches, I: SnipIter<T>> Clone for Snips<T, I> {
    fn clone(&self) -> Self {
        Self {
            repeat: true,
            index: self.index,
            end: self.end,
            terminated: self.terminated,
            snip: self.snip.copy_snip(),
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

#[derive(Debug, Clone, Copy)]
pub enum Jump {
    Up(usize),
    Back(usize),
}

#[derive(Debug, Clone)]
pub enum Comm<T: Matches> {
    Matched,
    Match(T),
    MatchAny,
    Jump(Jump),
    Branch(Jump, Jump),
    Scope,
    CommitScope,
    KillScope,
    Tok(bool),
    Save,
    Unsave,
    StartLoop,
    EndLoop(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Loop(Loop),
    Call(usize),
    Scope(usize),
    Save {
        ip: usize,
        last_event: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loop {
    pub start: usize,
    pub count: usize,
}
impl Loop {
    pub fn new(start: usize) -> Self {
        Self { start, count: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub state: Vec<State>,
    pub saves: usize,
    last_event: Option<usize>,
    prev_thread: Option<usize>,
    next_thread: Option<usize>,
}
impl Thread {
    pub fn new(ip: usize) -> Self {
        Self {
            ip,
            state: Vec::with_capacity(8),
            saves: 0,
            last_event: None,
            prev_thread: None,
            next_thread: None,
        }
    }

    pub fn fork_to(&self, fork: &mut Thread) {
        fork.ip = self.ip;
        fork.state.clone_from(&self.state);
        fork.saves = self.saves;
        fork.last_event = self.last_event;
        fork.prev_thread = None;
        fork.next_thread = None;
    }

    pub fn rewind(&mut self) {
        while let Some(state) = self.state.last_mut() {
            if let State::Save { ip, last_event } = state {
                self.ip = *ip;
                self.last_event = *last_event;
                return;
            } else {
                self.state.pop();
            }
        }
    }

    pub fn reset(&mut self) {
        self.ip = 0;
        self.state.clear();
        self.saves = 0;
        self.last_event = None;
        self.prev_thread = None;
        self.next_thread = None;
    }

    pub fn dbg(&self) -> String {
        format!(
            "Thread(ip={}, saves={}, ev={:?}, prev={:?}, next={:?}, state={:?})",
            self.ip, self.saves, self.last_event, self.prev_thread, self.next_thread, self.state
        )
    }
}

#[derive(Debug)]
pub struct Threads {
    debug: bool,
    pool: Vec<Thread>,
    first: Option<usize>,
    index: Option<usize>,
    last: Option<usize>,
    next_free: Option<usize>,
}

impl Threads {
    pub fn new() -> Self {
        let me = Self {
            debug: false,
            pool: vec![Thread::new(0), Thread::new(0)],
            first: Some(0),
            index: Some(0),
            last: Some(0),
            next_free: Some(1),
        };
        me
    }

    pub fn at(&mut self, id: usize) -> &mut Thread {
        unsafe { self.pool.get_unchecked_mut(id) }
    }

    pub fn next(&mut self) -> Option<(usize, usize)> {
        if let Some(index) = self.index {
            let thread = self.at(index);
            let ip = thread.ip;
            self.index = thread.next_thread;
            Some((index, ip))
        } else {
            None
        }
    }

    pub fn kill_scope(&mut self, scope_id: usize) {
        let mut thread_id = self.first;
        while let Some(id) = thread_id {
            let thread = self.at(id);
            thread_id = thread.next_thread;
            if thread.state.contains(&State::Scope(scope_id)) {
                self.free(id);
            }
        }
    }

    pub fn free(&mut self, id: usize) {
        let thread = self.at(id);
        let prev_thread = thread.prev_thread;
        let next_thread = thread.next_thread;

        if let Some(prev) = prev_thread {
            self.at(prev).next_thread = next_thread;
        }
        if let Some(next) = next_thread {
            self.at(next).prev_thread = prev_thread;
        }

        if let Some(mut free) = self.next_free {
            while let Some(f) = self.at(free).next_thread {
                free = f;
            }
            self.at(free).next_thread = Some(id);
        } else {
            self.next_free = Some(id);
        }

        if let Some(first) = self.first
            && first == id
        {
            self.first = next_thread;
        }
        if let Some(index) = self.index
            && index == id
        {
            self.index = next_thread;
        }
        if let Some(last) = self.last
            && last == id
        {
            self.last = prev_thread;
        }

        self.at(id).reset();
        if self.debug {
            println!("    Freed Thread {}", id);
        }
    }

    pub fn fork(&mut self, id: usize, func: impl FnOnce(&mut Thread)) {
        let fork_id = match self.next_free {
            Some(free) => {
                self.next_free = self.at(free).next_thread;
                free
            }
            None => {
                let free = self.pool.len();
                let t = Thread::new(0);
                self.pool.push(t);
                free
            }
        };
        let [thread, fork] = unsafe { self.pool.get_disjoint_unchecked_mut([id, fork_id]) };
        thread.fork_to(fork);
        func(fork);
        if let Some(last) = self.last {
            fork.prev_thread = Some(last);
            self.at(last).next_thread = Some(fork_id);
        }
        self.last = Some(fork_id);
        if self.index.is_none() {
            self.index = Some(fork_id);
        }
        if self.debug {
            println!("    Forked {} to {} ->\n{}", id, fork_id, self.dbg());
        }
    }

    pub fn restart(&mut self) -> bool {
        self.index = self.first;
        self.index.is_some()
    }

    pub fn dbg(&self) -> String {
        let mut dbg = String::new();
        dbg.push_str(&format!(
            "    Threads(first={:?}, index={:?}, last={:?}, free={:?}, [\n",
            self.first, self.index, self.last, self.next_free,
        ));
        for (i, thread) in self.pool.iter().enumerate() {
            dbg.push_str(&format!("        {}: {}\n", i, thread.dbg()));
        }
        dbg.push_str("    ])\n");
        dbg
    }
}

pub struct Parser<T: Matches> {
    stat: Stat,
    debug: bool,
    comms: Vec<Comm<T>>,
    threads: Threads,
    next_scope: usize,
    // seen: HashSet<usize>,
    events: Vec<Event>,
    best_match: Option<Option<usize>>,
    first_event: Option<usize>,
}

impl<T: Matches> Parser<T> {
    pub fn new(mut comms: Vec<Comm<T>>) -> Self {
        comms.push(Comm::Matched);
        Self {
            stat: Stat::Running,
            debug: false,
            comms,
            threads: Threads::new(),
            next_scope: 0,
            // seen: HashSet::new(),
            events: Vec::with_capacity(8),
            best_match: None,
            first_event: None,
        }
    }

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
        self.threads.debug = self.debug;
    }

    pub fn new_scope(&mut self) -> usize {
        let s = self.next_scope;
        self.next_scope += 1;
        s
    }

    pub fn parse<I: SnipIter<T>>(&mut self, source: impl AsSnips<T, I>) -> Stat {
        let mut snips = source.as_snips(..);
        while let Some(snip) = snips.next()
            && self.stat == Stat::Running
        {
            self.take_snip(&snip);
        }
        match self.best_match {
            Some(mut event) => {
                let mut last: Option<usize> = None;
                while let Some(event_index) = event {
                    let ev = unsafe { self.events.get_unchecked_mut(event_index) };
                    ev.next = last;
                    last = event;
                    event = ev.prev;
                }
                self.first_event = last;
                self.stat = Stat::Matched;
            }
            None => self.stat = Stat::Failed,
        };
        // let mut event = self.best_match.unwrap_or_default();
        // while let Some(event_index) = event {
        //     self.ord_events.push(event_index);
        //     event = unsafe { self.events.get_unchecked(event_index).prev };
        // }
        // self.ord_events.reverse();
        self.stat
    }

    pub fn take_snip<I: SnipIter<T>>(&mut self, snip: &Snip<T, I>) {
        while let Some((id, mut ip)) = self.threads.next()
            && self.stat == Stat::Running
        {
            loop {
                if self.debug {
                    println!(
                        "Thread {}: snipdex={}, snip={:?}, ip={}, comm={:?}",
                        id, snip.index, snip.value, ip, self.comms[ip]
                    );
                }
                match unsafe { self.comms.get_unchecked(ip) } {
                    Comm::Matched => {
                        let thread = self.threads.at(id);
                        thread.ip = ip;
                        self.best_match = Some(thread.last_event);
                        self.threads.free(id);
                        break;
                    }
                    Comm::Match(thing) => {
                        if let Some(value) = &snip.value {
                            let thread = self.threads.at(id);
                            if thing.matches(value) {
                                thread.ip = ip + 1;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                thread.rewind();
                            } else {
                                self.threads.free(id);
                            }
                        } else {
                            self.threads.free(id);
                        }
                        break;
                    }
                    Comm::MatchAny => {
                        self.threads.at(id).ip = ip + 1;
                        break;
                    }
                    &Comm::Jump(jump) => {
                        ip = match jump {
                            Jump::Up(add) => ip + add,
                            Jump::Back(sub) => ip - sub,
                        }
                    }
                    &Comm::Branch(b1, b2) => {
                        self.threads.fork(id, |thread| {
                            thread.ip = match b2 {
                                Jump::Up(add) => ip + add,
                                Jump::Back(sub) => ip - sub,
                            }
                        });
                        ip = match b1 {
                            Jump::Up(add) => ip + add,
                            Jump::Back(sub) => ip - sub,
                        };
                    }
                    Comm::Scope => {
                        let scope = self.new_scope();
                        self.threads.at(id).state.push(State::Scope(scope));
                        ip += 1;
                    }
                    Comm::CommitScope => {
                        if let Some(state) = self.threads.at(id).state.pop()
                            && let State::Scope(scope) = state
                        {
                            self.threads.kill_scope(scope);
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::KillScope => {
                        if let Some(State::Scope(scope)) = self.threads.at(id).state.last() {
                            let s = *scope;
                            self.threads.kill_scope(s);
                        } else {
                            println!("Tried to kill a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::Save => {
                        let thread = self.threads.at(id);
                        ip += 1;
                        thread.state.push(State::Save {
                            ip,
                            last_event: thread.last_event,
                        });
                        thread.saves += 1;
                    }
                    Comm::Unsave => {
                        let thread = self.threads.at(id);
                        if let Some(State::Save { .. }) = thread.state.pop() {
                            thread.saves -= 1;
                            ip += 1;
                        } else {
                            println!("Tried to unsave without a save");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    &Comm::Tok(start) => {
                        let thread = self.threads.at(id);
                        let event = Event {
                            start,
                            index: snip.index,
                            prev: thread.last_event,
                            next: None,
                        };
                        thread.last_event = Some(self.events.len());
                        self.events.push(event);
                        ip += 1;
                    }
                    Comm::StartLoop => {
                        let thread = self.threads.at(id);
                        thread.state.push(State::Loop(Loop::new(ip + 1)));
                        ip += 1;
                    }
                    &Comm::EndLoop(min, max) => {
                        let thread = self.threads.at(id);
                        if let Some(State::Loop(loo)) = thread.state.last_mut() {
                            loo.count += 1;
                            if loo.count == max {
                                thread.state.pop();
                                ip += 1;
                            } else {
                                let fork_ip = ip + 1;
                                ip = loo.start;
                                if loo.count >= min {
                                    self.threads.fork(id, |fork| {
                                        fork.ip = fork_ip;
                                        fork.state.pop();
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
                if self.debug {
                    println!("    {}", self.threads.at(id).dbg());
                }
            } // Command Loop
        } // Threads Loop

        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
        }
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
    comms.push(Comm::EndLoop(min, max));
    comms
}

pub fn run<T: Matches>(values: Vec<impl ToComms<T>>) -> Vec<Comm<T>> {
    let mut all = Vec::new();
    for value in values {
        all.extend(value.to_comms());
    }
    all
}

// pub fn till<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
//     let mut comms = vec![
//         Comm::Scope,
//         Comm::Branch(Jump::Up(1), Jump::Up(3)),
//         Comm::MatchAny,
//         Comm::Jump(Jump::Back(2)),
//     ];
//     comms.extend(values.to_comms());
//     comms.push(Comm::Commit);
//     comms
// }

pub fn till<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Save];
    comms.extend(values.to_comms());
    comms.push(Comm::Unsave);
    comms
}

pub fn alt<T: Matches>(values: Vec<impl ToComms<T>>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Scope];
    let mut branches: Vec<Vec<Comm<T>>> = Vec::new();
    let num_branches: usize = values.len();

    let mut total_len: usize = 0;
    for (i, value) in values.into_iter().enumerate() {
        let branch = value.to_comms();
        if i == num_branches - 1 {
            total_len += branch.len();
        } else {
            total_len += branch.len() + 1;
        }
        branches.push(branch);
    }

    let mut num_branches_left: usize = num_branches - 2;
    let mut len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i != num_branches - 1 {
            len += branch.len() + 1;
            comms.push(Comm::Branch(
                Jump::Up(1),
                Jump::Up(len + num_branches_left + 1),
            ));
            num_branches_left -= 1;

            total_len -= branch.len() + 1;
            branch.push(Comm::Jump(Jump::Up(total_len + 1)));
        }
    }

    for branch in branches {
        comms.extend(branch);
    }

    comms.push(Comm::CommitScope);
    comms
}
