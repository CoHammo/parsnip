use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

impl<T: Default> Index<usize> for LinkedVec<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.nodes.get_unchecked(index) }
    }
}

impl<T: Default> IndexMut<usize> for LinkedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.nodes.get_unchecked_mut(index) }
    }
}

#[derive(Debug, Clone)]
pub struct Node<T: Default> {
    pub value: T,
    pub prev: usize,
    pub next: usize,
    pub refs: usize,
}

impl<T: Default> Node<T> {
    pub fn new() -> Self {
        Self {
            value: T::default(),
            prev: 0,
            next: 0,
            refs: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkedVec<T: Default> {
    pub nodes: Vec<Node<T>>,
    use_refs: bool,
    iter_back: bool,
    pub first: usize,
    pub index: usize,
    pub last: usize,
    next_free: usize,
}

impl<T: Default> LinkedVec<T> {
    fn make() -> Self {
        let mut me = Self {
            nodes: Vec::new(),
            use_refs: false,
            iter_back: false,
            first: 0,
            index: 0,
            last: 0,
            next_free: 0,
        };
        me.nodes.push(Node::new());
        me
    }

    pub fn new() -> Self {
        Self::make()
    }

    pub fn with_refs() -> Self {
        let mut me = Self::make();
        me.iter_back = true;
        me.use_refs = true;
        me
    }

    pub fn get_multi<const N: usize>(&mut self, indices: [usize; N]) -> [&mut T; N] {
        unsafe {
            let mut arr: MaybeUninit<[&mut T; N]> = MaybeUninit::uninit();
            let arr_ptr = arr.as_mut_ptr();
            for i in 0..N {
                let idx = indices.get_unchecked(i);
                arr_ptr.cast::<&mut T>().add(i).write(&mut self[*idx].value);
            }
            arr.assume_init()
        }
    }

    pub fn take(&mut self) -> Self {
        Self {
            nodes: std::mem::take(&mut self.nodes),
            use_refs: self.use_refs,
            iter_back: self.iter_back,
            first: self.first,
            index: self.index,
            last: self.last,
            next_free: self.next_free,
        }
    }

    pub fn next(&mut self) -> Option<(usize, &mut T)> {
        let index = self.index;
        if index != 0 {
            match self.iter_back {
                true => self.index = self[index].prev,
                false => self.index = self[index].next,
            }
            Some((index, &mut self[index].value))
        } else {
            None
        }
    }

    pub fn restart_index(&mut self) -> bool {
        self.index = match self.iter_back {
            true => self.last,
            false => self.first,
        };
        self.index != 0
    }

    pub fn reverse(&mut self) {
        self.iter_back = !self.iter_back;
    }

    fn take_free(&mut self) -> usize {
        match self.next_free {
            0 => {
                let index = self.nodes.len();
                self.nodes.push(Node::new());
                index
            }
            index => {
                self.next_free = self[index].next;
                index
            }
        }
    }

    pub fn push(&mut self, with: impl FnOnce(&mut T)) -> Option<usize> {
        if !self.use_refs {
            let index = self.take_free();
            if self.first == 0 {
                self.first = index;
            }
            if self.index == 0 {
                self.index = index;
            }
            if self.last != 0 {
                let last = self.last;
                self[index].prev = last;
                self[last].next = index;
            }
            self.last = index;
            with(&mut self[index].value);
            Some(index)
        } else {
            None
        }
    }

    pub fn ref_push(&mut self, prev: usize, with: impl FnOnce(&mut T)) -> Option<usize> {
        if self.use_refs {
            let index = self.take_free();
            let node = &mut self[index];
            node.refs = 1;
            node.prev = prev;
            with(&mut node.value);
            Some(index)
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) {
        if !self.use_refs {
            let prev = self[index].prev;
            let next = self[index].next;

            self[prev].next = next;
            self[next].prev = prev;
            if self.first == index {
                self.first = next;
            }
            if self.index == index {
                self.index = next;
            }
            if self.last == index {
                self.last = prev;
            }

            self.free(index);
        }
    }

    pub fn ref_pop(&mut self, index: usize) -> Option<(usize, &T)> {
        if self.use_refs {
            let prev = self[index].prev;
            if self[index].refs == 1 {
                self.free(index);
            } else {
                self[index].refs -= 1;
            }
            Some((prev, &self[index].value))
        } else {
            None
        }
    }

    fn free(&mut self, index: usize) {
        self[index].prev = 0;
        self[index].next = self.next_free;
        self[index].refs = 0;
        self.next_free = index;
    }

    pub fn unref(&mut self, mut index: usize) {
        if self.use_refs {
            while index != 0 {
                let idx = index;
                if self[idx].refs == 1 {
                    index = self[idx].prev;
                    self.free(idx);
                } else {
                    self[idx].refs -= 1;
                    break;
                }
            }
        }
    }

    pub fn upref(&mut self, index: usize) {
        self[index].refs += 1;
    }
}
