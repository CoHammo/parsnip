use super::super::*;

parser!(Rep rep {
    parser: Parser => inner: Box<Parser>,
    min: Option<usize> => mmin: usize,
    max: Option<usize> => mmax: usize,
    count: usize = 0,
    end_byte: usize = 0,
} {
    mmin = match min {
        Some(num) => match num {
            0 => 1,
            _ => num,
        },
        None => 1,
    };
    mmax = match max {
        Some(num) => {
            if num > 0 && num < mmin {
                mmin
            } else {
                num
            }
        }
        None => {
            if min.is_none() {
                0
            } else {
                mmin
            }
        }
    };
    inner = Box::new(parser);
});

impl Clone for Rep {
    fn clone(&self) -> Self {
        Rep::new(*self.inner.clone(), Some(self.mmin), Some(self.mmax))
    }
}

impl CharParser for Rep {
    fn take_char(&mut self, chars: &mut ParseChars) -> Stat {
        freshen!(self, chars.char);
        match self.inner.take_char(chars) {
            Stat::Matched(end_byte) => {
                self.count += 1;
                self.end_byte = end_byte;
                self.base.add_tokens(self.inner.take_tokens());
                self.inner.reset();
                if self.count == self.mmax {
                    self.base.stat = Stat::Matched(end_byte);
                } else if self.count >= self.mmin {
                    self.base.stat = Stat::PossibleMatch(end_byte);
                }
            }
            Stat::Failed => {
                if self.count >= self.mmin {
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
            if self.count >= self.mmin {
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
