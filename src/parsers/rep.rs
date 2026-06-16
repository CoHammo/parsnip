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
        freshen!(self, ch);
        match self.inner.take_char(&ch) {
            Stat::Matched(end_byte) => {
                self.count += 1;
                self.end_byte = end_byte;
                self.base.add_tokens(self.inner.take_tokens());
                self.inner.reset();
                if self.count == self.the_max {
                    self.base.stat = Stat::Matched(end_byte);
                } else if self.count >= self.the_min {
                    self.base.stat = Stat::PossibleMatch(end_byte);
                }
            }
            Stat::Failed => {
                if self.count >= self.the_min {
                    self.base.stat = Stat::Matched(self.end_byte);
                } else {
                    self.base.stat = Stat::Failed;
                }
            }
            stat => self.base.stat = stat,
        }
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.inner.fresh() {
            if let Stat::PossibleMatch(end_byte) = self.base.stat {
                self.base.stat = Stat::Matched(end_byte);
            } else {
                self.base.stat = Stat::Failed;
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
                self.base.add_tokens(self.inner.take_tokens());
                self.inner.reset();
                self.base.stat = Stat::Matched(self.end_byte);
            } else {
                self.base.stat = Stat::Failed;
            };
        }
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
        self.count = 0;
        self.end_byte = 0;
    }

    fn string(&self) -> String {
        format!("Rep({})", self.inner.string())
    }
}
