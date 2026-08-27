use super::scopes::Scope;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Empty,
    Loop(Loop),
    // Call(usize),
    Save {
        ip: usize,
        event: usize,
        scope: Scope,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub start: usize,
    pub count: usize,
}
impl Loop {
    pub fn new(start: usize) -> Self {
        Self { start, count: 0 }
    }
}

#[derive(Debug, Clone)]
struct StateNode {
    state: State,
    prev: u16,
    refs: u16,
}

impl StateNode {
    pub fn new(state: State, prev: u16) -> Self {
        Self {
            state,
            prev,
            refs: 1,
        }
    }

    pub fn empty() -> Self {
        Self {
            state: State::Empty,
            prev: 0,
            refs: 0,
        }
    }
}

#[derive(Debug)]
pub struct StateStack {
    stack: Vec<StateNode>,
    free: u16,
}

impl StateStack {
    pub fn new() -> Self {
        Self {
            stack: vec![StateNode::empty()],
            free: 0,
        }
    }

    fn at(&mut self, id: u16) -> &mut StateNode {
        unsafe { self.stack.get_unchecked_mut(id as usize) }
    }

    pub fn check(&self, id: u16) -> Option<&State> {
        if id != 0 {
            Some(unsafe { &self.stack.get_unchecked(id as usize).state })
        } else {
            None
        }
    }

    pub fn push_state(&mut self, state: State, prev: u16) -> u16 {
        if self.free == 0 {
            let id = self.stack.len() as u16;
            if id == u16::MAX {
                panic!("State Stack Overflow!!");
            }
            self.stack.push(StateNode::new(state, prev));
            id
        } else {
            let id = self.free;
            self.free = self.at(id).prev;
            let node = self.at(id);
            node.state = state;
            node.prev = prev;
            node.refs = 1;
            id
        }
    }

    pub fn before(&self, id: u16) -> u16 {
        unsafe { self.stack.get_unchecked(id as usize).prev }
    }

    pub fn pop(&mut self, id: u16) -> (u16, State) {
        if id != 0 {
            let node = self.at(id);
            let prev = node.prev;
            let state = node.state.clone();
            node.refs -= 1;
            if node.refs == 0 {
                self.at(id).prev = self.free;
                self.free = id;
            } else {
                self.at(prev).refs += 1;
            }
            (prev, state)
        } else {
            (0, State::Empty)
        }
    }

    pub fn upref(&mut self, id: u16) {
        self.at(id).refs += 1;
    }

    pub fn unref(&mut self, mut id: u16) {
        while id != 0 {
            let free = self.free;
            let node = self.at(id);
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

    pub fn edit(&mut self, id: u16) -> (u16, &mut State) {
        let node = self.at(id);
        if node.refs > 1 {
            node.refs -= 1;
            let prev = node.prev;
            let state = node.state.clone();
            self.at(prev).refs += 1;
            let branch_id = self.push_state(state, prev);
            (branch_id, &mut self.at(branch_id).state)
        } else {
            (id, &mut self.at(id).state)
        }
    }
}
