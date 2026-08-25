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
    prev: usize,
    refs: usize,
}

impl StateNode {
    pub fn new(state: State, prev: usize) -> Self {
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
    free: usize,
}

impl StateStack {
    pub fn new() -> Self {
        Self {
            stack: vec![StateNode::empty()],
            free: 0,
        }
    }

    fn at(&mut self, id: usize) -> &mut StateNode {
        unsafe { self.stack.get_unchecked_mut(id) }
    }

    pub fn check(&self, id: usize) -> Option<&State> {
        if id != 0 {
            Some(unsafe { &self.stack.get_unchecked(id).state })
        } else {
            None
        }
    }

    pub fn push_state(&mut self, state: State, prev: usize) -> usize {
        let mut id = self.free;
        if self.free == 0 {
            id = self.stack.len();
            self.stack.push(StateNode::new(state, prev));
        } else {
            self.free = self.at(id).prev;
            let node = self.at(id);
            node.state = state;
            node.prev = prev;
            node.refs = 1;
        }
        id
    }

    pub fn before(&self, id: usize) -> usize {
        unsafe { self.stack.get_unchecked(id).prev }
    }

    pub fn pop(&mut self, id: usize) -> (usize, State) {
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

    pub fn upref(&mut self, id: usize) {
        self.at(id).refs += 1;
    }

    pub fn unref(&mut self, mut id: usize) {
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

    pub fn edit(&mut self, id: usize) -> (usize, &mut State) {
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
