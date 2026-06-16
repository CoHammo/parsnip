use super::super::*;

// parser!(Not not {
//    parser: Parser => inner: Box<Parser>,
// } {
//     inner = Box::new(parser);
// });

#[derive(Debug)]
pub struct Not {
    pub base: BaseParser,
    inner: Box<Parser>,
}
pub fn not(parser: Parser) -> Parser {
    Parser::Not(Not::new(parser))
}
impl Not {
    pub fn new(parser: Parser) -> Self {
        let mut me = Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        };
        me.base.stat = Stat::PossibleMatch(0);
        me
    }
}

impl Clone for Not {
    fn clone(&self) -> Self {
        Not::new(*self.inner.clone())
    }
}

impl CharParser for Not {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch, {
            self.base.stat = Stat::PossibleMatch(ch.byte);
        });
        match self.inner.take_char(ch) {
            Stat::Running | Stat::PossibleMatch(_) => {}
            Stat::Matched(_) => self.base.stat = Stat::Failed,
            Stat::Failed => self.base.stat = Stat::Matched(self.base.start_byte),
        }
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(_) => self.base.stat = Stat::Failed,
            _ => self.base.stat = Stat::Matched(self.base.start_byte),
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            self.base.stat = Stat::PossibleMatch(0);
            self.inner.reset();
        }
    }

    fn string(&self) -> String {
        format!("Not({})", self.inner.string())
    }
}
