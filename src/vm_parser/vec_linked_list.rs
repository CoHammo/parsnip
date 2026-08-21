pub trait CopyTo {
    fn copy_to(&self, target: &mut Self);
}

#[derive(Debug)]
pub struct Link<T: Default + CopyTo> {
    ptr: *mut Node<T>,
    // pub index: usize,
}

impl<T: Default + CopyTo> Link<T> {
    pub fn new(ptr: *mut Node<T>) -> Self {
        Self { ptr }
    }

    pub fn get(&self) -> &mut Node<T> {
        unsafe { &mut *self.ptr }
    }

    pub fn val(&self) -> &mut T {
        &mut (unsafe { &mut *self.ptr }).value
    }
}

impl<T: Default + CopyTo> Clone for Link<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T: Default + CopyTo> Copy for Link<T> {}

impl<T: Default + CopyTo> PartialEq for Link<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

#[derive(Debug, Clone)]
pub struct Node<T: Default + CopyTo> {
    pub value: T,
    pub prev: Option<Link<T>>,
    pub next: Option<Link<T>>,
    pub refs: usize,
}

impl<T: Default + CopyTo> Node<T> {
    pub fn new() -> Self {
        Self {
            value: T::default(),
            prev: None,
            next: None,
            refs: 0,
        }
    }

    pub fn reset(&mut self) {
        self.prev = None;
        self.next = None;
        self.refs = 0;
    }
}

// pub struct VecLinkedListIter<'a, T: Default + CopyTo<T>> {
//     source: &'a mut VecLinkedList<T>,
//     index: Option<usize>,
// }

// impl<'a, T: Default + CopyTo<T>> VecLinkedListIter<'a, T> {
//     pub fn new(source: &'a mut VecLinkedList<T>, index: Option<usize>) -> Self {
//         Self { source, index }
//     }

//     pub fn next(&mut self) -> Option<(usize, &mut Link<T>)> {
//         if let Some(index) = self.index {
//             let iter_back = self.source.iter_back;
//             let link = &mut self.source.data[index];
//             match iter_back {
//                 true => self.index = link.prev,
//                 false => self.index = link.next,
//             }
//             Some((index, link))
//         } else {
//             None
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct VecLinkedList<T: Default + CopyTo> {
    pub data: Vec<Node<T>>,
    use_refs: bool,
    auto_link: bool,
    iter_back: bool,
    pub first: Option<Link<T>>,
    pub index: Option<Link<T>>,
    pub last: Option<Link<T>>,
    next_free: Option<Link<T>>,
}

impl<T: Default + CopyTo> VecLinkedList<T> {
    pub fn new(use_refs: bool) -> Self {
        Self {
            data: Vec::new(),
            use_refs,
            auto_link: true,
            iter_back: false,
            first: None,
            index: None,
            last: None,
            next_free: None,
        }
    }

    pub fn manually_linked(use_refs: bool, iter_back: bool) -> Self {
        Self {
            data: Vec::new(),
            use_refs,
            auto_link: false,
            iter_back,
            first: None,
            index: None,
            last: None,
            next_free: None,
        }
    }

    pub fn get_link(&mut self, index: usize) -> Link<T> {
        unsafe { Link::new(self.data.get_unchecked_mut(index)) }
    }

    pub fn at(&mut self, index: usize) -> &mut Node<T> {
        unsafe { self.data.get_unchecked_mut(index) }
    }

    pub fn take_data(&mut self) -> Self {
        Self {
            data: std::mem::take(&mut self.data),
            use_refs: self.use_refs,
            auto_link: self.auto_link,
            iter_back: self.iter_back,
            first: self.first,
            index: self.index,
            last: self.last,
            next_free: self.next_free,
        }
    }

    // fn take_free_index(&mut self) -> usize {
    //     match self.next_free {
    //         Some(index) => {
    //             self.next_free = self.at(index).next;
    //             index
    //         }
    //         None => {
    //             let index = self.data.len();
    //             self.data.push(Node::new());
    //             index
    //         }
    //     }
    // }

    fn take_free_node(&mut self) -> Link<T> {
        match self.next_free {
            Some(link) => {
                self.next_free = link.get().next;
                link
            }
            None => {
                let index = self.data.len();
                self.data.push(Node::new());
                self.get_link(index)
            }
        }
    }

    // pub fn next(&mut self) -> Option<(usize, &mut T)> {
    //     if let Some(index) = self.index {
    //         let node = unsafe { self.data.get_unchecked_mut(index) };
    //         match self.iter_back {
    //             true => self.index = node.prev,
    //             false => self.index = node.next,
    //         }
    //         Some((index, &mut node.value))
    //     } else {
    //         None
    //     }
    // }

    pub fn next(&mut self) -> Option<Link<T>> {
        if let Some(link) = self.index {
            let node = link.get();
            match self.iter_back {
                true => self.index = node.prev,
                false => self.index = node.next,
            }
            Some(link)
        } else {
            None
        }
    }

    pub fn restart_index(&mut self) -> bool {
        self.index = match self.iter_back {
            true => self.last,
            false => self.first,
        };
        self.index.is_some()
    }

    pub fn reverse(&mut self) {
        self.iter_back = !self.iter_back;
    }

    fn push_auto_link(&mut self, link: &Link<T>) {
        let node = link.get();
        if self.use_refs {
            node.refs += 1;
        }
        if self.first.is_none() {
            self.first = Some(*link);
        }
        if self.index.is_none() {
            self.index = Some(*link);
        }
        if let Some(last) = self.last {
            node.prev = Some(last);
            last.get().next = Some(*link);
        }
        self.last = Some(*link);
    }

    pub fn push(&mut self) -> Option<Link<T>> {
        if self.auto_link {
            let link = self.take_free_node();
            self.push_auto_link(&link);
            Some(link)
        } else {
            None
        }
    }

    pub fn copy(&mut self, orig: &Link<T>) -> Option<Link<T>> {
        if self.auto_link {
            let copy = self.take_free_node();
            orig.val().copy_to(copy.val());
            self.push_auto_link(&copy);
            Some(copy)
        } else {
            None
        }
    }

    pub fn push_with_links(
        &mut self,
        prev: Option<Link<T>>,
        next: Option<Link<T>>,
    ) -> Option<Link<T>> {
        if !self.auto_link {
            let link = self.take_free_node();
            let node = link.get();
            if self.use_refs {
                node.refs = 1;
            }
            node.prev = prev;
            node.next = next;
            Some(link)
        } else {
            None
        }
    }

    pub fn copy_with_links(
        &mut self,
        orig: &Link<T>,
        prev: Option<Link<T>>,
        next: Option<Link<T>>,
    ) -> Option<Link<T>> {
        if !self.auto_link {
            let copy = self.take_free_node();
            orig.val().copy_to(copy.val());
            let node = copy.get();
            if self.use_refs {
                node.refs = 1;
            }
            node.prev = prev;
            node.next = next;
            Some(copy)
        } else {
            None
        }
    }

    pub fn remove(&mut self, link: Link<T>) {
        if self.use_refs {
            self.unref(link);
        } else {
            self.free_links(&link);
            self.free(link);
        }
    }

    fn free_links(&mut self, link: &Link<T>) {
        let node = link.get();
        if let Some(pre) = node.prev {
            pre.get().next = node.next;
        }
        if let Some(nxt) = node.next {
            nxt.get().prev = node.prev;
        }

        if self.first == Some(*link) {
            self.first = node.next;
        }
        if self.index == Some(*link) {
            self.index = node.next;
        }
        if self.last == Some(*link) {
            self.last = node.prev;
        }
    }

    fn free(&mut self, link: Link<T>) {
        let node = link.get();
        node.reset();
        node.next = self.next_free;
        self.next_free = Some(link);
    }

    fn unref(&mut self, link: Link<T>) {
        let mut next_link = Some(link);
        while let Some(l) = next_link {
            let node = l.get();
            if node.refs == 1 {
                next_link = node.prev;
                self.free(link);
            } else {
                node.refs -= 1;
                break;
            }
        }
    }

    pub fn add_ref(&mut self, link: &Link<T>) {
        if self.use_refs {
            link.get().refs += 1;
        }
    }
}
