#[derive(Debug, Clone, Copy)]
pub struct Scopes {
    scopes: u64,
    refs: [u8; 64],
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            scopes: 0,
            refs: [0; 64],
        }
    }

    pub fn next_scope(&mut self) -> u8 {
        let mut id: u8 = 0;
        for i in 0..64 {
            if self.refs[i] == 0 {
                self.refs[i] = 1;
                id = i as u8;
                break;
            } else if i == 63 {
                panic!("Parsing Scopes Overflowed!");
            }
        }
        self.scopes |= 1u64 << id;
        id
    }

    pub fn kill_scope(&mut self, id: u8, unref: bool) {
        self.scopes &= !(1u64 << id);
        if unref {
            self.refs[id as usize] -= 1;
        }
    }

    pub fn is_dead(&self, scope: u64) -> bool {
        if scope == 0 {
            false
        } else {
            (scope & self.scopes) != scope
        }
    }

    pub fn upref_scopes(&mut self, mut scopes: u64) {
        while scopes != 0 {
            let id = scopes.trailing_zeros();
            // println!("uprefing scope {}", id);
            self.refs[id as usize] += 1;
            scopes &= !(1u64 << id);
        }
    }

    pub fn unref_scopes(&mut self, mut scopes: u64) {
        while scopes != 0 {
            let id = scopes.trailing_zeros();
            // println!("unrefing scope {}", id);
            self.refs[id as usize] -= 1;
            scopes &= !(1u64 << id);
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
}
