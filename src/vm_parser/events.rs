#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub start: bool,
    pub index: usize,
    prev: usize,
    next: usize,
    refs: usize,
}

impl Event {
    pub fn new(start: bool, index: usize, prev: usize) -> Self {
        Self {
            start,
            index,
            prev,
            next: 0,
            refs: 1,
        }
    }

    pub fn empty() -> Self {
        Self {
            start: false,
            index: 0,
            prev: 0,
            next: 0,
            refs: 0,
        }
    }
}

#[derive(Debug)]
pub struct EventsBuilder {
    stack: Vec<Event>,
    free: usize,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self {
            stack: vec![Event::empty()],
            free: 0,
        }
    }

    fn at(&mut self, index: usize) -> &mut Event {
        unsafe { self.stack.get_unchecked_mut(index) }
    }

    pub fn push(&mut self, start: bool, index: usize, prev: usize) -> usize {
        let mut id = self.free;
        if self.free == 0 {
            id = self.stack.len();
            self.stack.push(Event::new(start, index, prev));
        } else {
            self.free = self.at(id).next;
            let event = self.at(id);
            event.start = start;
            event.index = index;
            event.prev = prev;
            event.next = 0;
            event.refs = 1;
        }
        id
    }

    pub fn upref(&mut self, id: usize) {
        self.at(id).refs += 1;
    }

    pub fn unref(&mut self, mut id: usize) {
        while id != 0 {
            let free = self.free;
            let event = self.at(id);
            if event.refs == 1 {
                let next = event.prev;
                event.refs = 0;
                event.prev = 0;
                event.next = free;
                self.free = id;
                id = next;
            } else {
                event.refs -= 1;
                break;
            }
        }
    }

    pub fn build_from(&mut self, mut id: usize) -> Events {
        let mut last = 0;
        let mut len = 0;
        while id != 0 {
            let event = self.at(id);
            event.next = last;
            last = id;
            id = event.prev;
            len += 1;
        }

        Events::new(std::mem::take(&mut self.stack), last, len)
    }
}

#[derive(Debug)]
pub struct Events {
    stack: Vec<Event>,
    first: usize,
    index: usize,
    valid_len: usize,
}

impl Events {
    pub fn new(stack: Vec<Event>, first: usize, valid_len: usize) -> Self {
        Self {
            stack,
            first,
            index: first,
            valid_len,
        }
    }

    pub fn empty() -> Self {
        Self {
            stack: Vec::new(),
            first: 0,
            index: 0,
            valid_len: 0,
        }
    }

    pub fn valid_len(&self) -> usize {
        self.valid_len
    }

    pub fn total_len(&self) -> usize {
        self.stack.len()
    }

    fn at(&self, index: usize) -> &Event {
        unsafe { self.stack.get_unchecked(index) }
    }

    pub fn next(&mut self) -> Option<&Event> {
        if self.index != 0 {
            let index = self.index;
            self.index = self.at(index).next;
            Some(self.at(index))
        } else {
            None
        }
    }
}
