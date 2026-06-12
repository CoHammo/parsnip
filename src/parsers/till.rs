use super::super::*;

parser!(Till till {
    parser: Parser => inner: Box<Parser>,
    match_end: Option<bool> => match_finish: bool,
} {
    match_finish = match_end.unwrap_or(false);
    inner = Box::new(parser);
});

impl Clone for Till {
    fn clone(&self) -> Self {
        Till::new(*self.inner.clone(), Some(self.match_finish))
    }
}

impl CharParser for Till {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        match self.inner.take_char(ch) {
            Stat::Matched(end_byte) => {
                let toks = self.inner.take_tokens();
                self.add_tokens(toks);
                self.stat = Stat::Matched(end_byte)
            }
            Stat::Failed => {
                self.inner.reset();
            }
            stat => self.stat = stat,
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.stat = Stat::Matched(end_byte);
            }
            _ => {
                if self.match_finish {
                    self.stat = Stat::Matched(ch.next_byte())
                } else {
                    self.stat = Stat::Failed;
                }
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Till({})", self.inner.string())
    }
}
