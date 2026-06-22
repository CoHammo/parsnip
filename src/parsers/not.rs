use super::super::*;

#[derive(Debug)]
pub struct Not<T: PI> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
}

impl<T: PI> Not<T> {
    pub fn new(parser: Parser<T>) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        }
    }

    fn look(&mut self, item: &IterItem<T>) {
        let mut peeks = item.peeks();
        while let Some(it) = peeks.next() {
            match self.inner.take(&it) {
                Stat::Running => {}
                Stat::Matched(_) => {
                    self.base.stat = Stat::Failed;
                    break;
                }
                Stat::Failed => {
                    self.base.stat = Stat::Matched(item.index());
                    break;
                }
            }
        }
    }
}

impl<T: PI> Clone for Not<T> {
    fn clone(&self) -> Self {
        Not::new(*self.inner.clone())
    }
}

impl<T: PI> ItemParser<T> for Not<T> {
    fn take(&mut self, item: &IterItem<T>) -> Stat {
        freshen!(self, item, {
            self.look(item);
        });
        self.base.stat
    }

    fn finish(&mut self, item: &IterItem<T>) -> Stat {
        match self.inner.finish(item) {
            Stat::Matched(_) => self.base.stat = Stat::Failed,
            _ => self.base.stat = Stat::Matched(item.index()),
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            self.inner.reset();
        }
    }

    fn string(&self) -> String {
        format!("Not({})", self.inner.string())
    }
}
