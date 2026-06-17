use super::super::*;

// parser!(Run run {
//     parsers: Vec<Parser> => inners: Box<[(Parser, Option<usize>)]>,
//     => len: usize,
//     inner_index: usize = 0,
//     inner_end_index: usize = 1,
// } {
//     inners = parsers.into_iter().map(|p| (p, None)).collect();
//     len = inners.len();
// });

#[derive(Debug)]
pub struct Run {
    pub base: BaseParser,
    inners: Box<[(Parser, Option<usize>)]>,
    len: usize,
    inner_index: usize,
    inner_end_index: usize,
}
pub fn run(parsers: Vec<Parser>) -> Parser {
    Parser::Run(Run::new(parsers))
}
impl Run {
    pub fn new(parsers: Vec<Parser>) -> Self {
        let len = parsers.len();
        let mut me = Self {
            base: BaseParser::new(),
            inners: parsers.into_iter().map(|p| (p, None)).collect(),
            len,
            inner_index: 0,
            inner_end_index: 0,
        };
        me.next_end();
        me
    }

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
                Stat::PossibleMatch(_) => {
                    if self.inner_index == self.inner_end_index {
                        self.next_end();
                    }
                    go = false;
                }
                Stat::Matched(_) => {
                    if self.inner_index == self.len - 1 {
                        return false;
                    }
                }
                Stat::Failed => {
                    self.base.stat = Stat::Failed;
                    go = false;
                }
            }
        }
        return !go;
    }

    fn set_end(&mut self, index: usize) {
        if index < self.len - 1 {
            self.inner_end_index = index + 1;
            self.next_end();
        }
    }

    fn next_end(&mut self) {
        while self.inner_end_index < self.len {
            let p = &mut self.inners[self.inner_end_index];
            p.0.reset();
            p.1 = None;
            self.inner_end_index += 1;
            if let Stat::PossibleMatch(_) = p.0.stat() {
                continue;
            } else {
                break;
            }
        }
    }

    fn collect_tokens(&mut self) {
        for parser in self.inners.iter_mut() {
            self.base.add_tokens(parser.0.take_tokens());
        }
    }
}

impl Clone for Run {
    fn clone(&self) -> Self {
        Run::new(self.inners.clone().into_iter().map(|p| p.0).collect())
    }
}

impl CharParser for Run {
    fn take_char(&mut self, chars: &mut ParseChars) -> Stat {
        freshen!(self, chars.char);
        for index in self.inner_index..self.inner_end_index {
            match self.inners.get_mut(index) {
                Some(parser) => match parser.0.take_char(chars) {
                    Stat::Running => {}
                    Stat::PossibleMatch(end_byte) => {
                        let mut should_break = false;
                        if let Some(prev_end_byte) = parser.1 {
                            if end_byte != prev_end_byte {
                                should_break = true;
                            }
                        } else {
                            should_break = true;
                        }
                        parser.1 = Some(end_byte);
                        self.set_end(index);
                        if index == self.len - 1 {
                            self.base.stat = Stat::PossibleMatch(end_byte);
                        }
                        if should_break {
                            break;
                        }
                    }
                    Stat::Matched(end_byte) => {
                        let mut should_break = false;
                        if let Some(prev_end_byte) = parser.1 {
                            if end_byte != prev_end_byte {
                                should_break = true;
                            }
                        }
                        if index == self.inner_index {
                            if !self.next_inner() {
                                self.collect_tokens();
                                self.base.stat = Stat::Matched(end_byte);
                            }
                        } else if index == self.inner_end_index - 1 {
                            self.next_end();
                        }
                        if should_break {
                            break;
                        }
                    }
                    Stat::Failed => {
                        self.inner_end_index = index;
                        if index == self.inner_index {
                            self.base.stat = Stat::Failed;
                        }
                        break;
                    }
                },
                None => self.base.stat = Stat::Failed,
            }
        }
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        for index in self.inner_index..self.inner_end_index {
            match self.inners.get_mut(index) {
                Some(parser) => match parser.0.finish(ch) {
                    Stat::Matched(end_byte) => {
                        self.collect_tokens();
                        if index == self.len - 1 {
                            self.base.stat = Stat::Matched(end_byte);
                        }
                    }
                    _ => {
                        self.base.stat = Stat::Failed;
                        break;
                    }
                },
                None => {
                    self.base.stat = Stat::Failed;
                    break;
                }
            }
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            for p in &mut self.inners {
                p.0.reset();
                p.1 = None;
            }
            self.inner_index = 0;
            self.inner_end_index = 0;
            self.next_end();
        }
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
