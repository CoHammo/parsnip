use super::vec_linked_list::*;

impl CopyTo for Event {
    fn copy_to(&self, target: &mut Self) {
        target.start = self.start;
        target.index = self.index;
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Event {
    pub start: bool,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct EventsBuilder {
    list: VecLinkedList<Event>,
}

impl EventsBuilder {
    pub fn new() -> Self {
        Self {
            list: VecLinkedList::manually_linked(true, true),
        }
    }

    pub fn push(&mut self, start: bool, index: usize, prev: Option<Link<Event>>) -> Link<Event> {
        let link = self.list.push_with_links(prev, None).unwrap();
        let event = link.val();
        event.start = start;
        event.index = index;
        link
    }

    pub fn unref(&mut self, link: Option<Link<Event>>) {
        if let Some(l) = link {
            self.list.remove(l);
        }
    }

    pub fn add_ref(&mut self, link: Option<Link<Event>>) {
        if let Some(l) = link {
            self.list.add_ref(&l);
        }
    }

    pub fn build_from(&mut self, link: &Link<Event>) -> Events {
        let mut last_link: Option<Link<Event>> = None;
        let mut next_link = Some(*link);
        while let Some(l) = next_link {
            let node = l.get();
            node.next = last_link;
            last_link = Some(l);
            next_link = node.prev;
        }

        Events::new(self.list.take_data(), last_link, Some(*link))
    }
}

#[derive(Debug, Clone)]
pub struct Events {
    list: VecLinkedList<Event>,
}

impl Events {
    pub fn new(
        mut list: VecLinkedList<Event>,
        first: Option<Link<Event>>,
        last: Option<Link<Event>>,
    ) -> Self {
        list.first = first;
        list.index = first;
        list.last = last;
        list.reverse();
        let me = Self { list };
        me
    }

    pub fn empty() -> Self {
        Self {
            list: VecLinkedList::new(false),
        }
    }

    pub fn len(&self) -> usize {
        self.list.data.len()
    }

    pub fn next(&mut self) -> Option<Event> {
        self.list.next().map(|link| *link.val())
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
