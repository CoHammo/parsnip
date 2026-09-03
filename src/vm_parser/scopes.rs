#[derive(Debug, Clone, Copy)]
pub struct Scopes {
    scopes: u64,
    index: u8,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: 0,
            index: 0,
        }
    }

    pub fn next_scope(&mut self) -> u8 {
        let id = self.index;
        if self.index == 63 {
            self.index = 0;
        } else {
            self.index += 1;
        }
        self.scopes |= 1u64 << id;
        id
    }

    pub fn kill_scope(&mut self, id: u8) {
        self.scopes &= !(1u64 << id);
    }

    pub fn is_dead(&self, scope: u64) -> bool {
        (scope & self.scopes) != scope
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scope {
    val: u64,
    last: u8,
}

impl Scope {
    pub fn new() -> Self {
        Self { val: 0, last: 0 }
    }

    pub fn val(&self) -> u64 {
        self.val
    }

    pub fn last_id(&self) -> u8 {
        self.last
    }

    pub fn add_scope(&mut self, id: u8) {
        self.val |= 1u64 << id;
        self.last = id;
    }

    pub fn last_scope(&self) -> Option<u8> {
        if self.val != 0 { Some(self.last) } else { None }
    }

    pub fn pop_scope(&mut self) -> Option<u8> {
        if self.val != 0 {
            let bit = 1u64 << self.last;
            self.val &= !bit;
            if self.val != 0 {
                let last = self.last;
                let mask = u64::MAX << last;
                let mut temp = (!mask & self.val).leading_zeros() as u8;
                if temp == 64 {
                    temp = (mask & self.val).leading_zeros() as u8;
                }
                let id = 63u8 - temp;
                self.last = id;
                Some(last)
            } else {
                Some(self.last)
            }
        } else {
            None
        }
    }
}
