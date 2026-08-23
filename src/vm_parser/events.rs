use super::linked_vec::*;

#[derive(Default, Debug, Clone, Copy)]
pub struct Event {
    pub start: bool,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct EventsBuilder {
    list: LinkedVec<Event>,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self {
            list: LinkedVec::with_refs(),
        }
    }

    pub fn add(&mut self, start: bool, index: usize, prev: usize) -> usize {
        self.list
            .ref_push(prev, |event| {
                event.start = start;
                event.index = index;
            })
            .unwrap()
    }

    pub fn upref(&mut self, id: usize) {
        self.list.upref(id);
    }

    pub fn unref(&mut self, id: usize) {
        self.list.unref(id);
    }

    pub fn build_from(&mut self, id: usize) -> Events {
        let mut last = 0;
        let mut index = id;
        let mut len = 0;
        while index != 0 {
            len += 1;
            let node = &mut self.list[index];
            node.next = last;
            last = index;
            index = node.prev;
        }
        Events::new(self.list.take(), last, id, len)
    }
}

#[derive(Debug, Clone)]
pub struct Events {
    list: LinkedVec<Event>,
    valid_len: usize,
}

impl Events {
    pub fn new(mut list: LinkedVec<Event>, first: usize, last: usize, valid_len: usize) -> Self {
        list.first = first;
        list.index = first;
        list.last = last;
        list.reverse();
        Self { list, valid_len }
    }

    pub fn empty() -> Self {
        Self {
            list: LinkedVec::new(),
            valid_len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.valid_len
    }

    pub fn total_len(&self) -> usize {
        self.list.nodes.len()
    }

    pub fn next(&mut self) -> Option<Event> {
        self.list.next().map(|(_, event)| *event)
    }

    pub fn restart(&mut self) {
        self.list.restart_index();
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct EventPart {
//     pub start: bool,
//     pub index: usize,
//     prev: Option<usize>,
//     next: Option<usize>,
// }

// impl EventPart {
//     pub fn new(start: bool, index: usize, prev: Option<usize>) -> Self {
//         Self {
//             start,
//             index,
//             prev,
//             next: None,
//         }
//     }

//     pub fn get(&self) -> Event {
//         Event {
//             start: self.start,
//             index: self.index,
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub struct Event {
//     pub start: bool,
//     pub index: usize,
// }

// #[derive(Debug, Clone)]
// pub struct EventsBuilder {
//     list: Vec<EventPart>,
// }

// impl EventsBuilder {
//     pub fn new() -> Self {
//         Self {
//             list: Vec::with_capacity(16),
//         }
//     }

//     pub fn push(&mut self, start: bool, index: usize, prev: Option<usize>) -> usize {
//         let id = self.list.len();
//         self.list.push(EventPart::new(start, index, prev));
//         id
//     }

//     pub fn build_from(&mut self, id: usize) -> Events {
//         let mut last_id: Option<usize> = None;
//         let mut event_id = Some(id);
//         while let Some(i) = event_id {
//             let event = unsafe { self.list.get_unchecked_mut(i) };
//             event.next = last_id;
//             last_id = event_id;
//             event_id = event.prev;
//         }
//         Events::new(std::mem::take(&mut self.list), last_id)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Events {
//     list: Vec<EventPart>,
//     first: Option<usize>,
//     index: Option<usize>,
// }

// impl Events {
//     pub fn new(events: Vec<EventPart>, first: Option<usize>) -> Self {
//         Self {
//             list: events,
//             first,
//             index: first,
//         }
//     }

//     pub fn empty() -> Self {
//         Self {
//             list: Vec::new(),
//             first: None,
//             index: None,
//         }
//     }

//     pub fn len(&self) -> usize {
//         self.list.len()
//     }

//     pub fn next(&mut self) -> Option<Event> {
//         if let Some(i) = self.index {
//             let event = unsafe { self.list.get_unchecked(i) };
//             self.index = event.next;
//             Some(event.get())
//         } else {
//             None
//         }
//     }

//     pub fn reset(&mut self) {
//         self.index = self.first;
//     }
// }
