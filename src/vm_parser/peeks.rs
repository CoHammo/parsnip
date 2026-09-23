use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct PeekStack {
    scopes: Vec<Peek>,
    free: u16,
}

impl PeekStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Peek::new(0, true)],
            free: 0,
        }
    }

    fn allocate(&mut self, parent_id: u16, positive: bool) -> u16 {
        let child_id: u16;
        if self.free == 0 {
            if self.scopes.len() == u16::MAX as usize {
                panic!("Peek Scope Overlow!!");
            }
            child_id = self.scopes.len() as u16 * 2;
            self.scopes.push(Peek::new(parent_id, positive));
        } else {
            child_id = self.free;
            self.free = self[child_id].next;
            self[child_id].renew(parent_id, positive);
        }
        child_id
    }

    fn add_child(&mut self, parent_id: u16, child_id: u16) {
        let scope = &mut self[parent_id];
        if scope.child == 0 {
            scope.child = child_id;
        } else {
            let mut sib = scope.child;
            while self[sib].next != 0 {
                sib = self[sib].next;
            }
            self[sib].next = child_id;
            self[child_id].prev = sib;
        }
    }

    pub fn add_peek(&mut self, parent_id: u16, positive: bool) -> u16 {
        let child_id = self.allocate(parent_id, positive);
        if parent_id != 0 {
            self.add_child(parent_id, child_id);
        }
        child_id
    }

    fn kill_peek_line(&mut self, id: u16) -> u16 {
        let peek = &mut self[id];
        peek.stat = PeekStat::Kill;
        let next_id = peek.next;
        let mut child_id = peek.child;
        while child_id != 0 {
            child_id = self.kill_peek_line(child_id);
        }
        next_id
    }

    fn remove_peek_line(&mut self, id: u16) -> u16 {
        let peek = &mut self[id];
        peek.stat = PeekStat::Remove;
        let next_id = peek.next;
        let mut child_id = peek.child;
        while child_id != 0 {
            child_id = self.remove_peek_line(child_id);
        }
        next_id
    }

    pub fn commit_peek(&mut self, id: u16) {
        if id != 0 {
            if self[id].positive {
                self.remove_peek_line(id);
            } else {
                self.kill_peek_line(id);
            }
        }
    }

    pub fn pop_peek(&mut self, mut id: u16) -> u16 {
        let mut free = true;
        while id != 0 && self[id].stat == PeekStat::Remove {
            let peek = &mut self[id];
            let parent = peek.parent;
            if peek.refs == 1 {
                if free {
                    peek.refs = 0;
                    if peek.peek_refs == 0 {
                        self.free(id);
                    }
                }
            } else {
                free = false;
            }
            id = parent;
        }
        if id != 0 {
            self[id].refs += 1;
        }
        id
    }

    pub fn upref(&mut self, id: u16) {
        if id != 0 {
            if id % 2 == 0 {
                self[id].refs += 1;
            } else {
                self[id].peek_refs += 1;
            }
        }
    }

    pub fn unref(&mut self, mut id: u16) {
        while id != 0 {
            let peek = &mut self[id];
            if id % 2 == 0 {
                peek.refs -= 1;
                if peek.refs == 0 {
                    if peek.peek_refs == 0 {
                        id = self.free(id);
                    } else {
                        peek.stat = PeekStat::Kill;
                        id = 0;
                    }
                } else {
                    id = 0;
                }
            } else {
                peek.peek_refs -= 1;
                if peek.peek_refs == 0 {
                    if peek.refs == 0 {
                        id = self.free(id - 1);
                    } else {
                        if peek.stat == PeekStat::Waiting {
                            if peek.positive {
                                peek.stat = PeekStat::Kill;
                            } else {
                                peek.stat = PeekStat::Remove;
                            }
                        }
                        id = 0;
                    }
                } else {
                    id = 0;
                }
            }
        }
    }

    fn free(&mut self, id: u16) -> u16 {
        let peek = &mut self[id];
        let parent = peek.parent;
        if parent != 0 {
            let prev = peek.prev;
            let next = peek.next;
            if prev != 0 {
                self[prev].next = next;
            }
            if next != 0 {
                self[next].prev = prev;
            }
            if self[parent].child == id {
                self[parent].child = next;
            }
        }
        self[id].next = self.free;
        self.free = id;
        parent
    }
}

impl Index<u16> for PeekStack {
    type Output = Peek;

    fn index(&self, id: u16) -> &Self::Output {
        unsafe { self.scopes.get_unchecked((id / 2) as usize) }
    }
}

impl IndexMut<u16> for PeekStack {
    fn index_mut(&mut self, id: u16) -> &mut Self::Output {
        unsafe { self.scopes.get_unchecked_mut((id / 2) as usize) }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeekStat {
    Waiting,
    Kill,
    Remove,
}

#[derive(Debug)]
pub struct Peek {
    pub stat: PeekStat,
    positive: bool,
    parent: u16,
    prev: u16,
    next: u16,
    child: u16,
    refs: u16,
    peek_refs: u16,
}

impl Peek {
    pub fn new(parent: u16, positive: bool) -> Self {
        Self {
            stat: PeekStat::Waiting,
            positive,
            parent,
            prev: 0,
            next: 0,
            child: 0,
            refs: 1,
            peek_refs: 1,
        }
    }

    pub fn renew(&mut self, parent: u16, positive: bool) {
        self.stat = PeekStat::Waiting;
        self.positive = positive;
        self.parent = parent;
        self.prev = 0;
        self.next = 0;
        self.child = 0;
        self.refs = 1;
        self.peek_refs = 1;
    }
}
