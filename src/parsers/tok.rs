use super::super::*;

#[derive(Debug)]
pub struct Tok<T: PI> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
    tag: Tag,
}
impl<T: PI> Tok<T> {
    pub fn new(parser: Parser<T>, tag: Tag) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            tag,
        }
    }
}
pub fn tok<T: PI>(parser: Parser<T>, tag: Tag) -> Parser<T> {
    Parser::Tok(Tok::new(parser, tag))
}

impl<T: PI> Clone for Tok<T> {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.tag)
    }
}

impl<T: PI> Tok<T> {
    fn tokenize(&mut self, end: usize) {
        self.base.tokens = Some(vec![Token::new(
            self.tag,
            self.inner.start(),
            end,
            self.inner.take_tokens(),
        )]);
    }
}
impl<T: PI> ItemParser<T> for Tok<T> {
    fn take(&mut self, item: &IterItem<T>) -> Stat {
        match self.inner.take(item) {
            Stat::Running => {}
            Stat::Matched(end) => {
                self.tokenize(end);
                self.base.stat = Stat::Matched(end);
            }
            Stat::Failed => self.base.stat = Stat::Failed,
        };
        self.base.fresh = self.inner.fresh();
        self.base.stat
    }

    fn finish(&mut self, item: &IterItem<T>) -> Stat {
        match self.inner.finish(item) {
            Stat::Matched(end) => {
                self.tokenize(end);
                self.base.stat = Stat::Matched(end);
            }
            stat => self.base.stat = stat,
        };
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }
}
