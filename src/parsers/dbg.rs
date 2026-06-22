use super::super::*;

#[derive(Debug)]
pub struct Dbg<T: PI> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
}
impl<T: PI> Dbg<T> {
    pub fn new(parser: Parser<T>) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        }
    }
}
pub fn dbg<T: PI>(parser: Parser<T>) -> Parser<T> {
    Parser::Dbg(Dbg::new(parser))
}

impl<T: PI> Clone for Dbg<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: PI> ItemParser<T> for Dbg<T> {
    fn take(&mut self, item: &IterItem<T>) -> Stat {
        freshen!(self, item);
        self.base.stat = self.inner.take(item);
        println!(
            "{}: item={:?}, index={}, stat={:?}",
            self.inner.string(),
            item.value,
            item.index(),
            self.base.stat
        );
        self.base.stat
    }

    fn finish(&mut self, item: &IterItem<T>) -> Stat {
        self.base.stat = self.inner.finish(item);
        println!(
            "Finish {}: item={:?}, index={}, stat={:?}",
            self.inner.string(),
            item.value,
            item.index(),
            self.base.stat
        );
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
        println!("Reset {}", self.inner.string())
    }

    fn string(&self) -> String {
        format!("Dbg({:?})", self.inner.string())
    }
}
