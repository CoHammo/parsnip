use super::*;

#[derive(Debug)]
pub struct Tok<T: PItem> {
    pub base: BaseParser,
    inner: Box<Parser<T>>,
    tag: Tag,
}
impl<T: PItem> Tok<T> {
    pub fn new(parser: Parser<T>, tag: Tag) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            tag,
        }
    }
}

impl<T: PItem> Clone for Tok<T> {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.tag)
    }
}

impl<T: PItem> Tok<T> {
    fn tokenize(&mut self, end: usize) {
        self.base.tokens = Some(vec![Token::new(
            self.tag,
            self.inner.start(),
            end,
            self.inner.take_tokens(),
        )]);
    }
}
impl<T: PItem> ItemParser<T> for Tok<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
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

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
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

pub fn tok<T: PItem>(parser: Parser<T>, tag: Tag) -> Parser<T> {
    Parser::Tok(Tok::new(parser, tag))
}
