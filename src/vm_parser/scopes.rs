use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    free: u16,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new(0)],
            free: 0,
        }
    }

    fn allocate(&mut self, parent: u16) -> u16 {
        let child: u16;
        if self.free == 0 {
            if self.scopes.len() == u16::MAX as usize {
                panic!("Scope Overlow!!");
            }
            child = self.scopes.len() as u16;
            self.scopes.push(Scope::new(parent));
        } else {
            child = self.free;
            self.free = self[child].next;
            self[child].renew(parent);
        }
        child
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
    }

    pub fn add_scope(&mut self, parent: u16) -> u16 {
        let child = self.allocate(parent);
        if parent != 0 {
            self.add_child(parent, child);
        }
        child
    }

    fn kill_line(&mut self, id: u16) {
        self[id].alive = false;
        let mut child = self[id].child;
        while child != 0 {
            self.kill_line(child);
            child = self[child].next;
        }
    }

    pub fn kill_scope(&mut self, id: u16) {
        if id != 0 {
            self.kill_line(id);
        }
    }

    pub fn pop_scope(&mut self, id: u16) -> Option<u16> {
        if id != 0 {
            let parent = self[id].parent;
            self[id].refs -= 1;
            if self[id].refs == 0 {
                self.free(id);
            } else {
                if parent != 0 {
                    self[parent].refs += 1;
                }
            }
            Some(parent)
        } else {
            None
        }
    }

    pub fn upref(&mut self, id: u16) {
        if id != 0 {
            self[id].refs += 1;
        }
    }

    pub fn unref(&mut self, mut id: u16) {
        while id != 0 && self[id].refs == 1 {
            self[id].refs = 0;
            id = self.free(id);
        }
        if id != 0 {
            self[id].refs -= 1;
        }
    }

    fn free(&mut self, id: u16) -> u16 {
        let scope = &mut self[id];
        let parent = scope.parent;
        if parent != 0 {
            let prev = scope.prev;
            let next = scope.next;
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

impl Index<u16> for ScopeStack {
    type Output = Scope;

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
pub struct Scope {
    pub alive: bool,
    parent: u16,
    prev: u16,
    next: u16,
    child: u16,
    refs: u16,
}

impl Scope {
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

    pub fn renew(&mut self, parent: u16) {
        self.alive = true;
        self.parent = parent;
        self.prev = 0;
        self.next = 0;
        self.child = 0;
        self.refs = 1;
    }
}
