use super::{Scope, Stack, Var};
use std::ops::{Index, IndexMut};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadState {
    ip: u16,
    scope: u64,
    last_scope: u8,
    stack: u16,
    saves: u16,
    event: u32,
}

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: u16,
    // scope: u64,
    // last_scope: u8,
    pub scope: Scope,
    pub stack: u16,
    pub saves: u16,
    pub event: u32,
    prev: u16,
    next: u16,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            ip: 0,
            // scope: 0,
            // last_scope: 0,
            scope: Scope::new(),
            stack: 0,
            saves: 0,
            event: 0,
            prev: 0,
            next: 0,
        }
    }

    pub fn get_state(&self, ip: u16) -> ThreadState {
        ThreadState {
            ip,
            scope: self.scope.val(),
            last_scope: self.scope.last_id(),
            stack: self.stack,
            saves: self.saves,
            event: self.event,
        }
    }

    pub fn rewind(&mut self, state: &mut Stack) {
        while let Some(st) = state.last(self.stack) {
            if let &Var::Save { ip, event, scope } = st {
                self.ip = ip;
                self.scope = scope;
                self.event = event;
                return;
            } else {
                let (prev, _) = state.pop_stack(self.stack).unwrap();
                self.stack = prev;
            }
        }
    }

    // pub fn get_scope(&self) -> u64 {
    //     self.scope
    // }

    // pub fn get_last_scope_id(&self) -> u8 {
    //     self.last_scope
    // }

    // pub fn get_last_scope(&self) -> Option<u8> {
    //     if self.scope != 0 {
    //         Some(self.last_scope)
    //     } else {
    //         None
    //     }
    // }

    // pub fn add_scope(&mut self, id: u8) {
    //     self.scope |= 1u64 << id;
    //     self.last_scope = id;
    // }

    // pub fn pop_scope(&mut self) -> Option<u8> {
    //     if self.scope != 0 {
    //         let bit = 1u64 << self.last_scope;
    //         self.scope &= !bit;
    //         if self.scope != 0 {
    //             let last = self.last_scope;
    //             let mask = u64::MAX << last;
    //             let mut temp = (!mask & self.scope).leading_zeros() as u8;
    //             if temp == 64 {
    //                 temp = (mask & self.scope).leading_zeros() as u8;
    //             }
    //             let id = 63u8 - temp;
    //             self.last_scope = id;
    //             Some(last)
    //         } else {
    //             Some(self.last_scope)
    //         }
    //     } else {
    //         None
    //     }
    // }

    pub fn dbg(&self) -> String {
        format!(
            "Thread(ip={:?}, saves={}, event={:?})",
            self.ip, self.saves, self.event
        )
    }
}

#[derive(Debug)]
pub struct Threads {
    pool: Vec<Thread>,
    first: u16,
    index: u16,
    last: u16,
    free: u16,
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

    pub fn len(&self) -> u16 {
        self.pool.len() as u16
    }

    pub fn next_thread(&mut self) -> Option<(u16, u16)> {
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

    fn allocate(&mut self) -> u16 {
        if self.free == 0 {
            let id = self.pool.len() as u16;
            if id == u16::MAX {
                panic!("Thread Pool Overflow!!");
            }
            self.pool.push(Thread::new());
            id
        } else {
            let id = self.free;
            self.free = self[id].next;
            id
        }
    }

    pub fn fork_thread(&mut self, id: u16) -> &mut Thread {
        let fork_id = self.allocate();
        let [orig, fork] = unsafe {
            self.pool
                .get_disjoint_unchecked_mut([id as usize, fork_id as usize])
        };
        fork.ip = orig.ip;
        fork.scope = orig.scope;
        fork.stack = orig.stack;
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

    pub fn kill(&mut self, id: u16) {
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

impl Index<u16> for Threads {
    type Output = Thread;

    fn index(&self, index: u16) -> &Self::Output {
        unsafe { self.pool.get_unchecked(index as usize) }
    }
}

impl IndexMut<u16> for Threads {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        unsafe { self.pool.get_unchecked_mut(index as usize) }
    }
}
