use super::super::*;

parser!(Rep rep {
    parser: Parser => inner: Box<Parser>,
    min: Option<usize> => the_min: usize,
    max: Option<usize> => the_max: usize,
    count: usize = 0,
    end_byte: usize = 0,
} {
    the_min = match min {
        Some(num) => match num {
            0 => 1,
            _ => num,
        },
        None => 1,
    };
    the_max = match max {
        Some(num) => {
            if num > 0 && num < the_min {
                the_min
            } else {
                num
            }
        }
        None => {
            if min.is_none() {
                0
            } else {
                the_min
            }
        }
    };
    inner = Box::new(parser);
});

impl Clone for Rep {
    fn clone(&self) -> Self {
        Rep::new(*self.inner.clone(), Some(self.the_min), Some(self.the_max))
    }
}

impl CharParser for Rep {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        match self.inner.take_char(&ch) {
            Stat::Matched(end_byte) => {
                self.count += 1;
                self.end_byte = end_byte;
                let toks = self.inner.take_tokens();
                self.add_tokens(toks);
                self.inner.reset();
                if self.count == self.the_max {
                    self.stat = Stat::Matched(end_byte);
                } else if self.count >= self.the_min {
                    self.stat = Stat::HasMatch(end_byte);
                }
            }
            Stat::Failed => {
                if self.count >= self.the_min {
                    self.stat = Stat::Matched(self.end_byte);
                } else {
                    self.stat = Stat::Failed;
                }
            }
            stat => self.stat = stat,
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.inner.fresh() {
            if let Stat::HasMatch(end_byte) = self.stat {
                self.stat = Stat::Matched(end_byte);
            } else {
                self.stat = Stat::Failed;
            }
        } else {
            match self.inner.finish(ch) {
                Stat::Matched(end_byte) => {
                    self.count += 1;
                    self.end_byte = end_byte;
                }
                _ => {}
            }
            if self.count >= self.the_min {
                let toks = self.inner.take_tokens();
                self.add_tokens(toks);
                self.inner.reset();
                self.stat = Stat::Matched(self.end_byte);
            } else {
                self.stat = Stat::Failed;
            };
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
        self.count = 0;
        self.end_byte = 0;
    }

    fn string(&self) -> String {
        format!("Rep({})", self.inner.string())
    }
}
