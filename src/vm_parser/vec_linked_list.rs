use std::ops::{Index, IndexMut};

impl<T: Default + CopyTo<T>> Index<usize> for VecLinkedList<T> {
    type Output = Link<T>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl<T: Default + CopyTo<T>> IndexMut<usize> for VecLinkedList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Link<T> {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

pub trait CopyTo<T: Default> {
    fn copy_to(&self, target: &mut T);
}

#[derive(Debug, Clone)]
pub struct Link<T: Default + CopyTo<T>> {
    pub value: T,
    pub prev: Option<usize>,
    pub next: Option<usize>,
    pub refs: usize,
}

impl<T: Default + CopyTo<T>> Link<T> {
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

pub struct VecLinkedListIter<'a, T: Default + CopyTo<T>> {
    source: &'a mut VecLinkedList<T>,
    index: Option<usize>,
}

impl<'a, T: Default + CopyTo<T>> VecLinkedListIter<'a, T> {
    pub fn new(source: &'a mut VecLinkedList<T>, index: Option<usize>) -> Self {
        Self { source, index }
    }

    pub fn next(&mut self) -> Option<(usize, &mut Link<T>)> {
        if let Some(index) = self.index {
            let iter_back = self.source.iter_back;
            let link = &mut self.source[index];
            match iter_back {
                true => self.index = link.prev,
                false => self.index = link.next,
            }
            Some((index, link))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct VecLinkedList<T: Default + CopyTo<T>> {
    pub data: Vec<Link<T>>,
    use_refs: bool,
    auto_link: bool,
    iter_back: bool,
    pub first: Option<usize>,
    pub index: Option<usize>,
    pub last: Option<usize>,
    next_free: Option<usize>,
}

impl<T: Default + CopyTo<T>> VecLinkedList<T> {
    pub fn new(capacity: usize, use_refs: bool) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            use_refs,
            auto_link: true,
            iter_back: false,
            first: None,
            index: None,
            last: None,
            next_free: None,
        }
    }

    pub fn manually_linked(capacity: usize, use_refs: bool, iter_back: bool) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            use_refs,
            auto_link: false,
            iter_back,
            first: None,
            index: None,
            last: None,
            next_free: None,
        }
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

    fn take_free_index(&mut self) -> usize {
        match self.next_free {
            Some(idx) => {
                self.next_free = self[idx].next;
                idx
            }
            None => {
                let idx = self.data.len();
                self.data.push(Link::new());
                idx
            }
        }
    }

    pub fn next(&mut self) -> Option<(usize, &mut Link<T>)> {
        if let Some(index) = self.index {
            let link = unsafe { self.data.get_unchecked_mut(index) };
            match self.iter_back {
                true => self.index = link.prev,
                false => self.index = link.next,
            }
            Some((index, link))
        } else {
            None
        }
    }

    pub fn restart_index(&mut self) -> bool {
        self.index = self.first;
        self.index.is_some()
    }

    pub fn reverse(&mut self) {
        self.iter_back = !self.iter_back;
    }

    pub fn iter_from(&mut self, index: Option<usize>) -> VecLinkedListIter<'_, T> {
        VecLinkedListIter::new(self, index)
    }

    fn private_push(&mut self, index: usize) {
        let link = unsafe { self.data.get_unchecked_mut(index) };
        if self.use_refs {
            link.refs = 1;
        }
        if self.first.is_none() {
            self.first = Some(index);
        }
        if let Some(last) = self.last {
            link.prev = Some(last);
            self[last].next = Some(index);
        }
        self.last = Some(index);
        if self.index.is_none() {
            self.index = Some(index);
        }
    }

    pub fn push(&mut self, edit: impl FnOnce(&mut T)) -> Option<usize> {
        if self.auto_link {
            let index = self.take_free_index();
            let link = unsafe { self.data.get_unchecked_mut(index) };
            edit(&mut link.value);
            self.private_push(index);
            Some(index)
        } else {
            None
        }
    }

    pub fn copy(&mut self, index: usize, edit: impl FnOnce(&mut T)) -> Option<usize> {
        if self.auto_link {
            let idx = self.take_free_index();
            let [orig, copy] = unsafe { self.data.get_disjoint_unchecked_mut([index, idx]) };
            orig.value.copy_to(&mut copy.value);
            edit(&mut copy.value);
            self.private_push(idx);
            Some(idx)
        } else {
            None
        }
    }

    pub fn push_with_links(
        &mut self,
        prev: Option<usize>,
        next: Option<usize>,
        edit: impl FnOnce(&mut T),
    ) -> Option<usize> {
        if !self.auto_link {
            let index = self.take_free_index();
            let link = unsafe { self.data.get_unchecked_mut(index) };
            edit(&mut link.value);
            if self.use_refs {
                link.refs = 1;
            }
            link.prev = prev;
            link.next = next;
            Some(index)
        } else {
            None
        }
    }

    pub fn copy_with_links(
        &mut self,
        index: usize,
        prev: Option<usize>,
        next: Option<usize>,
        edit: impl FnOnce(&mut T),
    ) -> Option<usize> {
        if !self.auto_link {
            let idx = self.take_free_index();
            let [orig, copy] = unsafe { self.data.get_disjoint_unchecked_mut([index, idx]) };
            orig.value.copy_to(&mut copy.value);
            edit(&mut copy.value);
            if self.use_refs {
                copy.refs = 1;
            }
            copy.prev = prev;
            copy.next = next;
            Some(idx)
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) {
        if self.use_refs {
            self.unref(index);
        } else {
            self.free(index);
        }
    }

    fn free(&mut self, index: usize) {
        if self.auto_link {
            let link = &self[index];
            let prev_link = link.prev;
            let next_link = link.next;

            if let Some(prev) = prev_link {
                self[prev].next = next_link;
            }
            if let Some(next) = next_link {
                self[next].prev = prev_link;
            }

            if self.first == Some(index) {
                self.first = next_link;
            }
            if self.index == Some(index) {
                self.index = next_link;
            }
            if self.last == Some(index) {
                self.last = prev_link;
            }
        }

        let link = unsafe { self.data.get_unchecked_mut(index) };
        link.reset();
        link.next = self.next_free;
        self.next_free = Some(index);
    }

    fn unref(&mut self, index: usize) {
        let mut idx = Some(index);
        while let Some(i) = idx {
            let link = unsafe { self.data.get_unchecked_mut(i) };
            if link.refs == 1 {
                idx = link.prev;
                self.free(i);
            } else {
                link.refs -= 1;
                break;
            }
        }
    }

    pub fn add_ref(&mut self, index: usize) {
        if self.use_refs {
            let link = &mut self[index];
            link.refs += 1;
        }
    }
}
