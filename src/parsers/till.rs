use super::super::*;

#[derive(Debug)]
pub struct Till<T: PItem> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
    match_finish: bool,
}
impl<T: PItem> Till<T> {
    pub fn new(parser: Parser<T>, match_end: bool) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            match_finish: match_end,
        }
    }
}
pub fn till<T: PItem>(parser: Parser<T>, match_end: bool) -> Parser<T> {
    Parser::Till(Till::new(parser, match_end))
}

impl<T: PItem> Clone for Till<T> {
    fn clone(&self) -> Self {
        Till::new(*self.inner.clone(), self.match_finish)
    }
}

impl<T: PItem> ItemParser<T> for Till<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        freshen!(self, item);
        match self.inner.take(item) {
            Stat::Running => {}
            Stat::Matched(end) => {
                self.base.add_tokens(self.inner.take_tokens());
                self.base.stat = Stat::Matched(end)
            }
            Stat::Failed => {
                self.inner.reset();
            }
        }
        self.base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        match self.inner.finish(item) {
            Stat::Matched(end) => {
                self.base.add_tokens(self.inner.take_tokens());
                self.base.stat = Stat::Matched(end);
            }
            _ => {
                if self.match_finish {
                    self.base.stat = Stat::Matched(item.index() + 1);
                } else {
                    self.base.stat = Stat::Failed;
                }
            }
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
        format!("Till({})", self.inner.string())
    }
}
