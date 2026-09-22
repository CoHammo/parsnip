pub trait Parses: Default + std::fmt::Debug + Clone + PartialEq {
    fn is_match(&self, other: &[u8]) -> bool;
    fn to_bytes(self) -> Box<[u8]>;
}

impl Parses for u8 {
    fn is_match(&self, other: &[u8]) -> bool {
        self == unsafe { other.get_unchecked(0) }
    }

    fn to_bytes(self) -> Box<[u8]> {
        Box::new([self])
    }
}

pub struct Snip<'a, T: Parses, I: Iterator<Item = T> + Clone> {
    pub value: Option<T>,
    pub index: u32,
    iter: Option<&'a Snips<T, I>>,
}

impl<'a, T: Parses, I: Iterator<Item = T> + Clone> Snip<'a, T, I> {
    pub fn new(value: T, index: u32, iter: Option<&'a Snips<T, I>>) -> Self {
        Self {
            value: Some(value),
            index,
            iter,
        }
    }

    pub fn empty(index: u32) -> Self {
        Self {
            value: None,
            index,
            iter: None,
        }
    }

    pub fn peek(&self) -> Option<Snips<T, I>> {
        self.iter.map(|iter| iter.copy())
    }
}

pub trait SnipsIter<T: Parses>: Iterator<Item = T> + Clone {}
impl<T: Parses, I: Iterator<Item = T> + Clone> SnipsIter<T> for I {}

pub struct Snips<T: Parses, I: Iterator<Item = T> + Clone> {
    index: u32,
    end: u32,
    terminated: bool,
    iter: I,
}

impl<T: Parses, I: Iterator<Item = T> + Clone> Snips<T, I> {
    pub fn new(iter: I) -> Self {
        Self {
            index: 0,
            end: u32::MAX,
            terminated: false,
            iter,
        }
    }

    pub fn range(mut iter: I, start: u32, mut end: u32) -> Self {
        if end <= start {
            end = start + 1;
        }
        for _ in 0..start {
            iter.next();
        }
        Self {
            index: start,
            end,
            terminated: false,
            iter,
        }
    }

    pub fn next(&mut self) -> Option<Snip<'_, T, I>> {
        if self.index < self.end
            && let Some(val) = self.iter.next()
        {
            let index = self.index;
            self.index += 1;
            let snip = Snip::new(val, index, Some(self));
            Some(snip)
        } else {
            if !self.terminated {
                self.terminated = true;
                Some(Snip::empty(self.index))
            } else {
                None
            }
        }
    }

    fn copy(&self) -> Self {
        Self {
            index: self.index,
            end: self.end,
            terminated: self.terminated,
            iter: self.iter.clone(),
        }
    }
}

pub trait AsSnips<T: Parses, I: Iterator<Item = T> + Clone> {
    fn snips(&self) -> Snips<T, I>;
    fn snips_range(&self, start: u32, end: u32) -> Snips<T, I>;
}

impl<'a> AsSnips<u8, std::str::Bytes<'a>> for &'a str {
    fn snips(&self) -> Snips<u8, std::str::Bytes<'a>> {
        let bytes = self.bytes();
        let snips = Snips::new(bytes);
        snips
    }

    fn snips_range(&self, start: u32, end: u32) -> Snips<u8, std::str::Bytes<'a>> {
        Snips::range(self.bytes(), start, end)
    }
}

impl<'a> AsSnips<u8, std::str::Bytes<'a>> for &'a String {
    fn snips(&self) -> Snips<u8, std::str::Bytes<'a>> {
        Snips::new(self.bytes())
    }

    fn snips_range(&self, start: u32, end: u32) -> Snips<u8, std::str::Bytes<'a>> {
        Snips::range(self.bytes(), start, end)
    }
}
