use super::super::*;

parser!(Tok tok {
    parser: Parser => inner: Box<Parser>,
    tag: Tag,
    save_value: Option<bool> => save: bool,
} {
    inner = Box::new(parser);
    save = save_value.unwrap_or(true);
});

impl Clone for Tok {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.tag.clone(), Some(self.save))
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
            Stat::Matched(end_byte) => {
                self.tokenize(end_byte);
                self.base.stat = Stat::Matched(end_byte);
            }
            stat => self.base.stat = stat,
        };
        self.base.fresh = self.inner.fresh();
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.tokenize(end_byte);
                self.base.stat = Stat::Matched(end_byte);
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
