use super::{types::*, vec_linked_list::*};

#[derive(Debug, Clone)]
pub struct Thread {
    pub ip: usize,
    pub state: Vec<State>,
    pub saves: usize,
    pub event: Option<usize>,
}
impl Thread {
    pub fn rewind(&mut self) {
        while let Some(state) = self.state.last_mut() {
            if let State::Save { ip, last_event } = state {
                self.ip = *ip;
                self.event = *last_event;
                return;
            } else {
                self.state.pop();
            }
        }
    }

    pub fn dbg(&self) -> String {
        format!(
            "Thread(ip={}, saves={}, ev={:?}, state={:?})",
            self.ip, self.saves, self.event, self.state
        )
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self {
            ip: 0,
            state: Vec::with_capacity(8),
            saves: 0,
            event: None,
        }
    }
}

impl CopyTo<Thread> for Thread {
    fn copy_to(&self, target: &mut Self) {
        target.ip = self.ip;
        target.state.clone_from(&self.state);
        target.saves = self.saves;
        target.event = self.event;
    }
}

#[derive(Debug, Clone)]
pub struct Threads {
    pool: VecLinkedList<Thread>,
}

impl Threads {
    pub fn new() -> Self {
        let mut me = Self {
            pool: VecLinkedList::new(4, false),
        };
        me.pool.push(|_| {});
        me
    }

    pub fn len(&self) -> usize {
        self.pool.data.len()
    }

    pub fn at(&mut self, id: usize) -> &mut Thread {
        &mut self.pool[id].value
    }

    pub fn next(&mut self) -> Option<(usize, usize)> {
        if let Some((index, thread)) = self.pool.next() {
            Some((index, thread.value.ip))
        } else {
            None
        }
    }

    pub fn restart(&mut self) -> bool {
        self.pool.restart_index()
    }

    pub fn fork(&mut self, id: usize, func: impl FnOnce(&mut Thread)) {
        self.pool.copy(id, func);
    }

    pub fn free(&mut self, id: usize) {
        self.pool.remove(id);
    }

    pub fn kill_scope(&mut self, scope: usize) {
        let mut index = self.pool.first;
        while let Some(idx) = index {
            let thread = &mut self.pool[idx];
            index = thread.next;
            if thread.value.state.contains(&State::Scope(scope)) {
                self.pool.remove(idx);
            }
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct Thread {
//     pub ip: usize,
//     pub state: Vec<State>,
//     pub saves: usize,
//     pub event: Option<usize>,
//     prev_thread: Option<usize>,
//     next_thread: Option<usize>,
// }
// impl Thread {
//     pub fn new(ip: usize) -> Self {
//         Self {
//             ip,
//             state: Vec::with_capacity(8),
//             saves: 0,
//             event: None,
//             prev_thread: None,
//             next_thread: None,
//         }
//     }

//     pub fn fork_to(&self, fork: &mut Thread) {
//         fork.ip = self.ip;
//         fork.state.clone_from(&self.state);
//         fork.saves = self.saves;
//         fork.event = self.event;
//         fork.prev_thread = None;
//         fork.next_thread = None;
//     }

//     pub fn rewind(&mut self) {
//         while let Some(state) = self.state.last_mut() {
//             if let State::Save { ip, last_event } = state {
//                 self.ip = *ip;
//                 self.event = *last_event;
//                 return;
//             } else {
//                 self.state.pop();
//             }
//         }
//     }

//     pub fn reset(&mut self) {
//         self.ip = 0;
//         self.state.clear();
//         self.saves = 0;
//         self.event = None;
//         self.prev_thread = None;
//         self.next_thread = None;
//     }

//     pub fn dbg(&self) -> String {
//         format!(
//             "Thread(ip={}, saves={}, ev={:?}, prev={:?}, next={:?}, state={:?})",
//             self.ip, self.saves, self.event, self.prev_thread, self.next_thread, self.state
//         )
//     }
// }

// #[derive(Debug)]
// pub struct Threads {
//     pub debug: bool,
//     pool: Vec<Thread>,
//     first: Option<usize>,
//     index: Option<usize>,
//     last: Option<usize>,
//     next_free: Option<usize>,
// }

// impl Threads {
//     pub fn new() -> Self {
//         let me = Self {
//             debug: false,
//             pool: vec![Thread::new(0), Thread::new(0)],
//             first: Some(0),
//             index: Some(0),
//             last: Some(0),
//             next_free: Some(1),
//         };
//         me
//     }

//     pub fn len(&self) -> usize {
//         self.pool.len()
//     }

//     pub fn at(&mut self, id: usize) -> &mut Thread {
//         unsafe { self.pool.get_unchecked_mut(id) }
//     }

//     pub fn next(&mut self) -> Option<(usize, usize)> {
//         if let Some(index) = self.index {
//             let thread = self.at(index);
//             let ip = thread.ip;
//             self.index = thread.next_thread;
//             Some((index, ip))
//         } else {
//             None
//         }
//     }

//     pub fn kill_scope(&mut self, scope_id: usize) {
//         let mut thread_id = self.first;
//         while let Some(id) = thread_id {
//             let thread = self.at(id);
//             thread_id = thread.next_thread;
//             if thread.state.contains(&State::Scope(scope_id)) {
//                 self.free(id);
//             }
//         }
//     }

//     pub fn free(&mut self, id: usize) {
//         let thread = self.at(id);
//         let prev_thread = thread.prev_thread;
//         let next_thread = thread.next_thread;

//         if let Some(prev) = prev_thread {
//             self.at(prev).next_thread = next_thread;
//         }
//         if let Some(next) = next_thread {
//             self.at(next).prev_thread = prev_thread;
//         }

//         if let Some(first) = self.first
//             && first == id
//         {
//             self.first = next_thread;
//         }
//         if let Some(index) = self.index
//             && index == id
//         {
//             self.index = next_thread;
//         }
//         if let Some(last) = self.last
//             && last == id
//         {
//             self.last = prev_thread;
//         }

//         let next_free = self.next_free;
//         let thread = self.at(id);
//         thread.reset();
//         thread.next_thread = next_free;
//         self.next_free = Some(id);

//         if self.debug {
//             println!("    Freed Thread {}", id);
//         }
//     }

//     pub fn fork(&mut self, id: usize, func: impl FnOnce(&mut Thread)) {
//         let fork_id = match self.next_free {
//             Some(free_id) => {
//                 self.next_free = self.at(free_id).next_thread;
//                 free_id
//             }
//             None => {
//                 let free_id = self.pool.len();
//                 let t = Thread::new(0);
//                 self.pool.push(t);
//                 free_id
//             }
//         };
//         let [thread, fork] = unsafe { self.pool.get_disjoint_unchecked_mut([id, fork_id]) };
//         thread.fork_to(fork);
//         func(fork);
//         if let Some(last) = self.last {
//             fork.prev_thread = Some(last);
//             self.at(last).next_thread = Some(fork_id);
//         }
//         self.last = Some(fork_id);
//         if self.index.is_none() {
//             self.index = Some(fork_id);
//         }
//         if self.debug {
//             println!("    Forked {} to {} ->\n{}", id, fork_id, self.dbg());
//         }
//     }

//     pub fn restart(&mut self) -> bool {
//         self.index = self.first;
//         self.index.is_some()
//     }

//     pub fn dbg(&self) -> String {
//         let mut dbg = String::new();
//         dbg.push_str(&format!(
//             "    Threads(first={:?}, index={:?}, last={:?}, free={:?}, [\n",
//             self.first, self.index, self.last, self.next_free,
//         ));
//         for (i, thread) in self.pool.iter().enumerate() {
//             dbg.push_str(&format!("        {}: {}\n", i, thread.dbg()));
//         }
//         dbg.push_str("    ])\n");
//         dbg
//     }
// }
