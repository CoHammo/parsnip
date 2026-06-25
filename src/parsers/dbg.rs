use super::super::*;

#[derive(Debug)]
pub struct Dbg<T: PItem> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
}
impl<T: PItem> Dbg<T> {
    pub fn new(parser: Parser<T>) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        }
    }
}
pub fn dbg<T: PItem>(parser: Parser<T>) -> Parser<T> {
    Parser::Dbg(Dbg::new(parser))
}

impl<T: PItem> Clone for Dbg<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: PItem> ItemParser<T> for Dbg<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
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

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
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
