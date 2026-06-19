use super::super::*;

parser!(Run run {
    parsers: Vec<Parser> => inners: Box<[Parser]>,
    => len: usize,
    inner_index: usize = 0,
    check_after_byte: usize = 0,
} {
    inners = parsers.into_boxed_slice();
    len = inners.len();
});

impl Clone for Run {
    fn clone(&self) -> Self {
        Run::new(self.inners.clone().to_vec())
    }
}

impl CharParser for Run {
    fn take_char(&mut self, ch: &Char) -> Stat {
        if self.base.fresh || ch.byte > self.check_after_byte {
            freshen!(self, ch);
            let parser = &mut self.inners[self.inner_index];
            match parser.take_char(ch) {
                Stat::Matched(end_byte) => {
                    self.base.add_tokens(parser.take_tokens());
                    if self.inner_index == self.len - 1 {
                        self.base.stat = Stat::Matched(end_byte);
                    } else {
                        self.inner_index += 1;
                        if end_byte < ch.byte {
                            if let Some(c) = ch.peeks().next() {
                                self.take_char(&c);
                            }
                        } else {
                            self.check_after_byte = end_byte;
                        }
                    }
                }
                Stat::Failed => self.base.stat = Stat::Failed,
                _ => {}
            }
        }
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        let parser = &mut self.inners[self.inner_index];
        match parser.finish(ch) {
            Stat::Matched(end_byte) => {
                self.base.add_tokens(parser.take_tokens());
                if self.inner_index == self.len - 1 {
                    self.base.stat = Stat::Matched(end_byte);
                } else if end_byte == ch.byte {
                    self.inner_index += 1;
                    self.finish(ch);
                } else {
                    self.base.stat = Stat::Failed;
                }
            }
            _ => {}
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            for index in 0..=self.inner_index {
                self.inners[index].reset();
            }
            self.inner_index = 0;
        }
    }

    fn string(&self) -> String {
        format!(
            "Run([{}])",
            self.inners
                .iter()
                .map(|p| p.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
