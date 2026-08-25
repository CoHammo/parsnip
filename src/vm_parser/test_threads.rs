use super::scopes::Scope;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub state: usize,
    pub scope: Scope,
    pub saves: usize,
    pub event: usize,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            ip: 0,
            state: 0,
            scope: Scope::new(),
            saves: 0,
            event: 0,
        }
    }

    pub fn dbg(&self) -> String {
        format!(
            "Thread(ip={}, saves={}, event={:?})",
            self.ip, self.saves, self.event
        )
    }
}

#[derive(Debug, Clone)]
pub struct Threads {
    threads: Vec<Thread>,
    next: Vec<Thread>,
    index: usize,
}

impl Threads {
    pub fn new() -> Self {
        Self {
            threads: vec![Thread::new()],
            next: Vec::new(),
            index: 0,
        }
    }

    pub fn next_thread(&mut self) -> Option<(usize, usize)> {
        if self.index < self.threads.len() {
            let index = self.index;
            self.index += 1;
            Some((index, self[index].ip))
        } else {
            None
        }
    }

    pub fn survive(&mut self, id: usize) {
        self.next.push(self[id].clone());
    }

    pub fn fork_thread(&mut self, id: usize) -> &mut Thread {
        let fork_id = self.threads.len();
        self.threads.push(self[id].clone());
        unsafe { self.threads.get_unchecked_mut(fork_id) }
    }

    pub fn restart(&mut self) -> bool {
        if self.next.is_empty() {
            false
        } else {
            self.index = 0;
            std::mem::swap(&mut self.threads, &mut self.next);
            self.next.clear();
            // println!("Threads after swap: {:#?}", self);
            true
        }
    }
}

impl Index<usize> for Threads {
    type Output = Thread;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.threads.get_unchecked(index) }
    }
}

impl IndexMut<usize> for Threads {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.threads.get_unchecked_mut(index) }
    }
}
