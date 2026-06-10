use super::super::*;

parser!(Run run {
    parsers: Vec<Parser> => inners: Box<[(Parser, Option<usize>)]>,
    => len: usize,
    inner_index: usize = 0,
    inner_end_index: usize = 1,
} {
    inners = parsers.into_iter().map(|p| (p, None)).collect();
    len = inners.len();
});

impl Run {
    fn next_inner(&mut self) -> bool {
        let mut go = true;
        while self.inner_index < self.len - 1 && go {
            self.inner_index += 1;
            match self.inners[self.inner_index].0.stat() {
                Stat::Running => {
                    if self.inner_index == self.inner_end_index {
                        self.inner_end_index += 1;
                    }
                    go = false;
                }
                Stat::HasMatch(_) => go = false,
                Stat::Matched(_) => {
                    if self.inner_index == self.len - 1 {
                        return false;
                    }
                }
                Stat::Failed => {
                    self.stat = Stat::Failed;
                    go = false;
                }
            }
        }
        return !go;
    }

    fn set_end(&mut self, index: usize) {
        if index < self.len - 1 {
            self.inner_end_index = index + 2;
            let p = &mut self.inners[self.inner_end_index - 1];
            p.0.reset();
            p.1 = None;
        }
    }

    fn next_end(&mut self) {
        if self.inner_end_index < self.len {
            let p = &mut self.inners[self.inner_end_index];
            p.0.reset();
            p.1 = None;
            self.inner_end_index += 1;
        }
    }
}

impl Clone for Run {
    fn clone(&self) -> Self {
        Run::new(self.inners.clone().into_iter().map(|p| p.0).collect())
    }
}

impl CharParser for Run {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        for index in self.inner_index..self.inner_end_index {
            match self.inners.get_mut(index) {
                Some(parser) => match parser.0.take_char(ch) {
                    Stat::Running => {}
                    Stat::HasMatch(end_byte) => {
                        parser.1 = Some(end_byte);
                        self.set_end(index);
                        if index == self.len - 1 {
                            self.stat = Stat::HasMatch(end_byte);
                        }
                        break;
                    }
                    Stat::Matched(end_byte) => {
                        let mut should_break = false;
                        if let Some(last_end_byte) = parser.1 {
                            if last_end_byte != end_byte {
                                should_break = true;
                            }
                        }
                        if index == self.inner_index {
                            let toks = parser.0.take_tokens();
                            self.add_tokens(toks);
                            if !self.next_inner() {
                                self.stat = Stat::Matched(end_byte);
                            }
                        } else if index == self.inner_end_index - 1 {
                            self.next_end();
                        }
                        if should_break {
                            break;
                        }
                    }
                    Stat::Failed => {
                        if index == self.inner_index {
                            self.stat = Stat::Failed;
                        }
                    }
                },
                None => self.stat = Stat::Failed,
            }
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.inner_index == self.len - 1 {
            if let Some(p) = self.inners.get_mut(self.inner_index) {
                match p.0.finish(ch) {
                    Stat::Matched(end_byte) => {
                        let toks = p.0.take_tokens();
                        self.add_tokens(toks);
                        self.stat = Stat::Matched(end_byte)
                    }
                    _ => self.stat = Stat::Failed,
                }
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        for p in &mut self.inners[..self.inner_end_index] {
            p.0.reset();
            p.1 = None;
        }
        self.inner_index = 0;
        self.inner_end_index = 1;
    }

    fn string(&self) -> String {
        format!(
            "Run([{}])",
            self.inners
                .iter()
                .map(|p| p.0.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
