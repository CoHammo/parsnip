use super::super::*;

#[derive(Debug)]
pub struct Run {
    pub base: BaseParser,
    inners: Box<[Parser]>,
    len: usize,
    index: usize,
    check_at_byte: usize,
}
impl Run {
    pub fn new(parsers: Vec<Parser>) -> Self {
        let len = parsers.len();
        Self {
            base: BaseParser::new(),
            inners: parsers.into_boxed_slice(),
            len,
            index: 0,
            check_at_byte: 0,
        }
    }
}
pub fn run(parsers: Vec<Parser>) -> Parser {
    Parser::Run(Run::new(parsers))
}

impl Clone for Run {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inners: self.inners.clone(),
            len: self.len,
            index: 0,
            check_at_byte: 0,
        }
    }
}

impl CharParser for Run {
    fn take_char(&mut self, ch: &Char) -> Stat {
        if ch.byte >= self.check_at_byte {
            freshen!(self, ch);
            let parser = &mut self.inners[self.index];
            match parser.take_char(ch) {
                Stat::Matched(byte) => {
                    self.base.add_tokens(parser.take_tokens());
                    if self.index == self.len - 1 {
                        self.base.stat = Stat::Matched(byte);
                    } else {
                        self.index += 1;
                        if byte == ch.byte {
                            self.take_char(ch);
                        } else {
                            self.check_at_byte = byte;
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
        let parser = &mut self.inners[self.index];
        match parser.finish(ch) {
            Stat::Matched(byte) => {
                self.base.add_tokens(parser.take_tokens());
                if self.index == self.len - 1 {
                    self.base.stat = Stat::Matched(byte);
                } else if byte == ch.byte {
                    self.index += 1;
                    self.finish(ch);
                } else {
                    self.base.stat = Stat::Failed;
                }
            }
            _ => self.base.stat = Stat::Failed,
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            for index in 0..=self.index {
                self.inners[index].reset();
            }
            self.index = 0;
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
