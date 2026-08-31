use super::Scope;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Var {
    Empty,
    Loop(Loop),
    // Call(usize),
    Save { ip: u16, event: u32, scope: Scope },
}

impl Var {
    pub fn loo(start: u16) -> Var {
        Var::Loop(Loop::new(start))
    }

    pub fn save(ip: u16, event: u32, scope: Scope) -> Var {
        Var::Save { ip, event, scope }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loop {
    pub start: u16,
    pub count: u32,
}
impl Loop {
    pub fn new(start: u16) -> Self {
        Self { start, count: 0 }
    }
}

#[derive(Debug, Clone)]
struct VarNode {
    var: Var,
    prev: u16,
    refs: u16,
}

impl VarNode {
    pub fn new(var: Var, prev: u16) -> Self {
        Self { var, prev, refs: 1 }
    }

    pub fn empty() -> Self {
        Self {
            var: Var::Empty,
            prev: 0,
            refs: 0,
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    stack: Vec<VarNode>,
    free: u16,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: vec![VarNode::empty()],
            free: 0,
        }
    }

    fn at_mut(&mut self, id: u16) -> &mut VarNode {
        unsafe { self.stack.get_unchecked_mut(id as usize) }
    }

    fn at(&self, id: u16) -> &VarNode {
        unsafe { self.stack.get_unchecked(id as usize) }
    }

    pub fn last(&self, id: u16) -> Option<&Var> {
        if id != 0 {
            Some(unsafe { &self.stack.get_unchecked(id as usize).var })
        } else {
            None
        }
    }

    pub fn push_stack(&mut self, var: Var, prev: u16) -> u16 {
        if self.free == 0 {
            let id = self.stack.len() as u16;
            if id == u16::MAX {
                panic!("State Stack Overflow!!");
            }
            self.stack.push(VarNode::new(var, prev));
            id
        } else {
            let id = self.free;
            self.free = self.at(id).prev;
            let node = self.at_mut(id);
            node.var = var;
            node.prev = prev;
            node.refs = 1;
            id
        }
    }

    pub fn before(&self, id: u16) -> u16 {
        unsafe { self.stack.get_unchecked(id as usize).prev }
    }

    pub fn pop_stack(&mut self, id: u16) -> Option<(u16, Var)> {
        if id != 0 {
            let node = self.at_mut(id);
            let prev = node.prev;
            let var = node.var.clone();
            node.refs -= 1;
            if node.refs == 0 {
                self.at_mut(id).prev = self.free;
                self.free = id;
            } else {
                self.at_mut(prev).refs += 1;
            }
            Some((prev, var))
        } else {
            None
        }
    }

    pub fn upref(&mut self, id: u16) {
        self.at_mut(id).refs += 1;
    }

    pub fn unref(&mut self, mut id: u16) {
        while id != 0 {
            let free = self.free;
            let node = self.at_mut(id);
            node.refs -= 1;
            if node.refs == 0 {
                let next = node.prev;
                node.prev = free;
                self.free = id;
                id = next;
            } else {
                break;
            }
        }
    }

    pub fn edit(&mut self, id: u16) -> Option<(u16, &mut Var)> {
        if id != 0 {
            let node = self.at_mut(id);
            if node.refs > 1 {
                node.refs -= 1;
                let prev = node.prev;
                let state = node.var.clone();
                self.at_mut(prev).refs += 1;
                let branch_id = self.push_stack(state, prev);
                Some((branch_id, &mut self.at_mut(branch_id).var))
            } else {
                Some((id, &mut self.at_mut(id).var))
            }
        } else {
            None
        }
    }

    pub fn contains(&self, mut id: u16, var: Var) -> bool {
        while id != 0 {
            let node = self.at(id);
            if node.var == var {
                return true;
            } else {
                id = node.prev;
            }
        }
        false
    }
}
