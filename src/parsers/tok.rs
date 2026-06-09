use super::super::*;

parser!(Tok tok {
    parser: Parser => inner: Box<Parser>,
    kind: Option<String>,
    save_value: Option<bool> => save: bool,
} {
    inner = Box::new(parser);
    save = save_value.unwrap_or(true);
});

impl Clone for Tok {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.kind.clone(), Some(self.save))
    }
}

impl Tok {
    fn tokenize(&mut self, from_string: &str, start_byte: usize, end_byte: usize) {
        self.tokens = Some(vec![Token::new(
            self.kind.clone(),
            match self.save {
                true => Some(&from_string[start_byte..end_byte]),
                false => None,
            },
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
                self.tokenize(ch.full_string, self.inner.start_byte(), end_byte);
                self.stat = Stat::Matched(end_byte);
            }
            stat => self.stat = stat,
        };
        self.fresh = self.inner.fresh();
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.tokenize(ch.full_string, self.inner.start_byte(), end_byte);
                self.stat = Stat::Matched(end_byte);
            }
            stat => self.stat = stat,
        };
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }
}
