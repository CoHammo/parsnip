use super::super::*;

parser!(Not not {
   inner: Box<Parser>,
} {});

impl Clone for Not {
    fn clone(&self) -> Self {
        Not::new(self.inner.clone())
    }
}

impl CharParser for Not {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        match self.inner.take_char(ch) {
            Stat::Running => {}
            Stat::HasMatch(_) | Stat::Matched(_) => self.stat = Stat::Failed,
            Stat::Failed => self.stat = Stat::Matched(ch.next_byte_offset()),
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::HasMatch(_) | Stat::Matched(_) => self.stat = Stat::Failed,
            _ => self.stat = Stat::Matched(ch.next_byte_offset()),
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Not({})", self.inner.string())
    }
}
