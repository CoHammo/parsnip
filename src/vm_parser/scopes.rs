use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct ScopeStack {
    scopes: Vec<Scope2>,
    free: u16,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope2::new(0)],
            free: 0,
        }
    }

    fn add_child(&mut self, parent: u16, child: u16) {
        let scope = &mut self[parent];
        if scope.child == 0 {
            scope.child = child;
        } else {
            let mut sib = scope.child;
            while self[sib].next != 0 {
                sib = self[sib].next;
            }
            self[sib].next = child;
            self[child].prev = sib;
        }
        self[child].parent = parent;
    }

    pub fn get_next_scope(&mut self, parent: u16) -> u16 {
        if self.free == 0 {
            if self.scopes.len() == u16::MAX as usize {
                panic!("Scope Overflow!!");
            }
            let child = self.scopes.len() as u16;
            self.scopes.push(Scope2::new(parent));
            if parent != 0 {
                self.add_child(parent, child);
            }
            child
        } else {
            let child = self.free;
            self.free = self[child].next;
            let ch = &mut self[child];
            ch.alive = true;
            ch.prev = 0;
            ch.next = 0;
            ch.child = 0;
            ch.refs = 1;
            if parent != 0 {
                self.add_child(parent, child);
            }
            child
        }
    }

    fn kill_lineage(&mut self, id: u16) {
        self[id].alive = false;
        let mut child = self[id].child;
        while child != 0 {
            self.kill_lineage(child);
            child = self[child].next;
        }
    }

    pub fn kill_scope(&mut self, id: u16) {
        if id != 0 {
            self.kill_lineage(id);
        }
    }

    pub fn remove_scope(&mut self, id: u16) {
        let parent = self[id].parent;
        let child = self[id].child;
        if parent != 0 {
            if child != 0 {
                self.add_child(parent, child);
            }
            self[parent].refs += self[id].refs - 1;
            let prev = self[id].prev;
            let next = self[id].next;
            self[prev].next = next;
            self[next].prev = prev;
            if self[parent].child == id {
                self[parent].child = next;
            } else {
                self[parent].child = 0;
            }
        }
        self[id].next = self.free;
        self.free = id;
    }

    pub fn pop_scope(&mut self, id: u16) -> Option<u16> {
        if id != 0 {
            let parent = self[id].parent;
            self[id].refs -= 1;
            if self[id].refs == 0 {
                if parent != 0 {
                    let next = self[id].next;
                    let prev = self[id].prev;
                    self[prev].next = next;
                    self[next].prev = prev;
                    if self[parent].child == id {
                        self[parent].child = next;
                    } else {
                        self[parent].child = 0;
                    }
                }
                self[id].next = self.free;
                self.free = id;
            } else {
                self[parent].refs += 1;
            }
            Some(parent)
        } else {
            None
        }
    }

    pub fn upref(&mut self, id: u16) {
        self[id].refs += 1;
    }

    pub fn unref(&mut self, mut id: u16) {
        while id != 0 && self[id].refs == 1 {
            self[id].refs = 0;
            let parent = self[id].parent;
            if parent != 0 {
                let prev = self[id].prev;
                let next = self[id].next;
                self[prev].next = next;
                self[next].prev = prev;
                if self[parent].child == id {
                    self[parent].child = next;
                } else {
                    self[parent].child = 0;
                }
            }
            self[id].next = self.free;
            self.free = id;
            id = self[id].parent;
        }
        self[id].refs -= 1;
    }
}

impl Index<u16> for ScopeStack {
    type Output = Scope2;

    fn index(&self, id: u16) -> &Self::Output {
        unsafe { self.scopes.get_unchecked(id as usize) }
    }
}

impl IndexMut<u16> for ScopeStack {
    fn index_mut(&mut self, id: u16) -> &mut Self::Output {
        unsafe { self.scopes.get_unchecked_mut(id as usize) }
    }
}

#[derive(Debug, Clone)]
pub struct Scope2 {
    pub alive: bool,
    parent: u16,
    prev: u16,
    next: u16,
    child: u16,
    refs: u16,
}

impl Scope2 {
    pub fn new(parent: u16) -> Self {
        Self {
            alive: true,
            parent,
            prev: 0,
            next: 0,
            child: 0,
            refs: 1,
        }
    }
}
