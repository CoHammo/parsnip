use super::super::*;

#[derive(Debug)]
pub struct It<T: PItem> {
    pub base: BaseParser,
    items: Box<[T]>,
    len: usize,
    index: usize,
}
impl<T: PItem> It<T> {
    pub fn new(value: impl Parses<T>) -> Self {
        let items = value.to_inner_store();
        let len = items.len();
        Self {
            base: BaseParser::new(),
            items,
            len,
            index: 0,
        }
    }
}

impl<T: PItem> Clone for It<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            items: self.items.clone(),
            len: self.len,
            index: 0,
        }
    }
}

impl<T: PItem> ItemParser<T> for It<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        if self.base.fresh {
            self.base.start = item.index();
            self.base.fresh = false;
        }
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else if item.value.matches(&self.items[self.index]) {
            self.index += 1;
            if self.index == self.len {
                self.base.stat = Stat::Matched(item.index() + 1);
            }
        } else {
            self.base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}({}), byte={}, index={}, stat={:?}",
        //     self.bytes,
        //     self.bytes[self.index - 1],
        //     byte.value,
        //     byte.index(),
        //     self.base.stat
        // );
        self.base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        if self.len == 0 {
            self.base.stat = Stat::Matched(item.index() + 1);
        } else {
            self.base.stat = Stat::Failed;
        };
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            self.index = 0;
        }
    }

    fn string(&self) -> String {
        format!("It({:?})", self.items)
    }
}

pub fn it<T: PItem>(value: impl Parses<T>) -> Parser<T> {
    Parser::It(It::new(value))
}
