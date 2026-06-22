use super::super::*;

#[derive(Debug)]
pub struct Not {
    pub base: BaseParser,
    inner: Box<Parser>,
}

impl Not {
    pub fn new(parser: Parser) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        }
    }

    fn look(&mut self, ch: &Char) {
        let mut peeks = ch.peeks();
        while let Some(c) = peeks.next() {
            match self.inner.take_char(&c) {
                Stat::Running => {}
                Stat::Matched(_) => {
                    self.base.stat = Stat::Failed;
                    break;
                }
                Stat::Failed => {
                    self.base.stat = Stat::Matched(ch.byte);
                    break;
                }
            }
        }
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
            self.look(ch);
        });
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(_) => self.base.stat = Stat::Failed,
            _ => self.base.stat = Stat::Matched(ch.byte),
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
