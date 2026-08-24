use super::{scopes::Scope, state::*};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub state: usize,
    pub scope: Scope,
    pub saves: usize,
    pub event: usize,
    prev: usize,
    next: usize,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            ip: 0,
            // state: Vec::new(),
            state: 0,
            scope: Scope::new(),
            saves: 0,
            event: 0,
            prev: 0,
            next: 0,
        }
    }

    // pub fn rewind(&mut self) {
    //     while let Some(state) = self.state.last() {
    //         if let &State::Save { ip, event, scope } = state {
    //             self.ip = ip;
    //             self.event = event;
    //             self.scope = scope;
    //             return;
    //         } else {
    //             self.state.pop();
    //         }
    //     }
    // }

    pub fn rewind(&mut self, state: &mut StateStack) {
        while let Some(st) = state.check(self.state) {
            if let &State::Save { ip, event, scope } = st {
                self.ip = ip;
                self.event = event;
                self.scope = scope;
                return;
            } else {
                let (prev, _) = state.pop(self.state);
                self.state = prev;
            }
        }
    }

    pub fn dbg(&self) -> String {
        format!(
            "Thread(ip={}, saves={}, ev={:?}",
            self.ip, self.saves, self.event
        )
    }
}

#[derive(Debug)]
pub struct Threads {
    pool: Vec<Thread>,
    first: usize,
    index: usize,
    last: usize,
    free: usize,
}

impl Threads {
    pub fn new() -> Self {
        Self {
            pool: vec![Thread::new(), Thread::new()],
            first: 1,
            index: 1,
            last: 1,
            free: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn next(&mut self) -> Option<(usize, usize)> {
        if self.index != 0 {
            let id = self.index;
            self.index = self[id].next;
            Some((id, self[id].ip))
        } else {
            None
        }
    }

    pub fn restart(&mut self) -> bool {
        self.index = self.first;
        self.index != 0
    }

    fn allocate(&mut self) -> usize {
        let mut id = self.free;
        match self.free {
            0 => {
                id = self.pool.len();
                self.pool.push(Thread::new());
            }
            _ => {
                self.free = self[id].next;
            }
        }
        id
    }

    pub fn fork(&mut self, id: usize) -> &mut Thread {
        let fork_id = self.allocate();
        let [orig, fork] = unsafe { self.pool.get_disjoint_unchecked_mut([id, fork_id]) };
        fork.ip = orig.ip;
        fork.state = orig.state;
        fork.scope = orig.scope;
        fork.saves = orig.saves;
        fork.event = orig.event;

        fork.next = 0;
        if self.first == 0 {
            self.first = fork_id;
        }
        if self.index == 0 {
            self.index = fork_id;
        }
        if self.last != 0 {
            self[fork_id].prev = self.last;
            let last = self.last;
            self[last].next = fork_id;
        }
        self.last = fork_id;
        &mut self[fork_id]
    }

    pub fn kill(&mut self, id: usize) {
        let prev = self[id].prev;
        let next = self[id].next;

        self[prev].next = next;
        self[next].prev = prev;
        if self.first == id {
            self.first = next;
        }
        if self.index == id {
            self.index = next;
        }
        if self.last == id {
            self.last = prev;
        }

        self[id].prev = 0;
        self[id].next = self.free;
        self.free = id;
    }
}

impl Index<usize> for Threads {
    type Output = Thread;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.pool.get_unchecked(index) }
    }
}

impl IndexMut<usize> for Threads {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.pool.get_unchecked_mut(index) }
    }
}
