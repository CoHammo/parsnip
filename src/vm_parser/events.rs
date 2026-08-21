#[derive(Debug, Clone, Copy)]
pub struct EventPart {
    pub start: bool,
    pub index: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

impl EventPart {
    pub fn new(start: bool, index: usize, prev: Option<usize>) -> Self {
        Self {
            start,
            index,
            prev,
            next: None,
        }
    }

    pub fn get(&self) -> Event {
        Event {
            start: self.start,
            index: self.index,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub start: bool,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct EventsBuilder {
    list: Vec<EventPart>,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self {
            list: Vec::with_capacity(16),
        }
    }

    pub fn push(&mut self, start: bool, index: usize, prev: Option<usize>) -> usize {
        let id = self.list.len();
        self.list.push(EventPart::new(start, index, prev));
        id
    }

    pub fn build(&mut self, id: usize) -> Events {
        let mut last_id: Option<usize> = None;
        let mut event_id = Some(id);
        while let Some(i) = event_id {
            let event = unsafe { self.list.get_unchecked_mut(i) };
            event.next = last_id;
            last_id = event_id;
            event_id = event.prev;
        }
        Events::new(std::mem::take(&mut self.list), last_id)
    }
}

#[derive(Debug, Clone)]
pub struct Events {
    list: Vec<EventPart>,
    first: Option<usize>,
    index: Option<usize>,
}

impl Events {
    pub fn new(events: Vec<EventPart>, first: Option<usize>) -> Self {
        Self {
            list: events,
            first,
            index: first,
        }
    }

    pub fn empty() -> Self {
        Self {
            list: Vec::new(),
            first: None,
            index: None,
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        if let Some(i) = self.index {
            let event = unsafe { self.list.get_unchecked(i) };
            self.index = event.next;
            Some(event.get())
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.index = self.first;
    }
}
