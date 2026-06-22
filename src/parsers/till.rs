use super::super::*;

#[derive(Debug)]
pub struct Till {
    pub base: BaseParser,
    inner: Box<Parser>,
    match_finish: bool,
}
impl Till {
    pub fn new(parser: Parser, match_end: bool) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
            match_finish: match_end,
        }
    }
}
pub fn till(parser: Parser, match_end: bool) -> Parser {
    Parser::Till(Till::new(parser, match_end))
}

impl Clone for Till {
    fn clone(&self) -> Self {
        Till::new(*self.inner.clone(), self.match_finish)
    }
}

impl CharParser for Till {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        match self.inner.take_char(ch) {
            Stat::Running => {}
            Stat::Matched(byte) => {
                self.base.add_tokens(self.inner.take_tokens());
                self.base.stat = Stat::Matched(byte)
            }
            Stat::Failed => {
                self.inner.reset();
            }
        }
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(byte) => {
                self.base.add_tokens(self.inner.take_tokens());
                self.base.stat = Stat::Matched(byte);
            }
            _ => {
                if self.match_finish {
                    self.base.stat = Stat::Matched(ch.next_byte());
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
