use super::super::*;

#[derive(Debug)]
pub struct Tok {
    pub base: BaseParser,
    inner: Box<Parser>,
    tag: Tag,
}
impl Tok {
    pub fn new(parser: Parser, tag: Tag) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            tag,
        }
    }
}
pub fn tok(parser: Parser, tag: Tag) -> Parser {
    Parser::Tok(Tok::new(parser, tag))
}

impl Clone for Tok {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.tag)
    }
}

impl Tok {
    fn tokenize(&mut self, end_byte: usize) {
        let start_byte = self.inner.start_byte();
        self.base.tokens = Some(vec![Token::new(
            self.tag,
            start_byte,
            end_byte,
            self.inner.take_tokens(),
        )]);
    }
}
impl CharParser for Tok {
    fn take_char(&mut self, ch: &Char) -> Stat {
        match self.inner.take_char(ch) {
            Stat::Running => {}
            Stat::Matched(byte) => {
                self.tokenize(byte);
                self.base.stat = Stat::Matched(byte);
            }
            Stat::Failed => self.base.stat = Stat::Failed,
        };
        self.base.fresh = self.inner.fresh();
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(byte) => {
                self.tokenize(byte);
                self.base.stat = Stat::Matched(byte);
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
