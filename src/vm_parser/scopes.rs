#[derive(Debug, Clone, Copy)]
pub struct Scopes {
    scopes: u64,
    id: u8,
    wrapped: bool,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: 0,
            id: 0,
            wrapped: false,
        }
    }

    pub fn next(&mut self) -> u8 {
        if self.id == 64 {
            self.wrapped = true;
            self.id = 0;
        }

        let mask = 1u64 << self.id;
        if self.wrapped && ((self.scopes & mask) != 0) {
            panic!("Parsing scopes overflowed at {}", self.id);
        }
        self.scopes |= mask;

        self.id += 1;
        self.id - 1
    }

    pub fn kill(&mut self, id: u8) {
        self.scopes &= !(1u64 << id);
    }

    pub fn kill_scopes(&mut self, scope: u64) {
        self.scopes &= !scope;
    }

    pub fn is_dead(&self, scope: u64) -> bool {
        if scope == 0 {
            false
        } else {
            (scope & self.scopes) != scope
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn add_scope(&mut self, id: u8) {
        self.val |= 1u64 << id;
        self.last = id;
    }

    pub fn last(&self) -> Option<u8> {
        if self.val != 0 { Some(self.last) } else { None }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.val != 0 {
            let mut mask = 1u64 << self.last;
            self.val &= !mask;

            if self.val != 0 {
                let last = self.last;
                let mut id = 63u8;
                mask = 1u64 << id;
                loop {
                    if (self.val & mask) != 0 {
                        self.last = id;
                        break;
                    } else {
                        id -= 1;
                        mask >>= 1;
                    }
                }
                Some(last)
            } else {
                Some(self.last)
            }
        } else {
            None
        }
    }

    pub fn diff(&self, other: u64) -> u64 {
        self.val ^ other
    }
}
