use super::super::*;

#[derive(Debug)]
pub struct Run<T: PItem> {
    pub base: BaseParser,
    inners: Box<[Parser<T>]>,
    len: usize,
    index: usize,
    check_at_index: usize,
}
impl<T: PItem> Run<T> {
    pub fn new(parsers: Vec<Parser<T>>) -> Self {
        let len = parsers.len();
        Self {
            base: BaseParser::new(),
            inners: parsers.into_boxed_slice(),
            len,
            index: 0,
            check_at_index: 0,
        }
    }
}

impl<T: PItem> Clone for Run<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inners: self.inners.clone(),
            len: self.len,
            index: 0,
            check_at_index: 0,
        }
    }
}

impl<T: PItem> ItemParser<T> for Run<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        if item.index() >= self.check_at_index {
            freshen!(self, item);
            let parser = &mut self.inners[self.index];
            match parser.take(item) {
                Stat::Matched(end) => {
                    self.base.add_tokens(parser.take_tokens());
                    if self.index == self.len - 1 {
                        self.base.stat = Stat::Matched(end);
                    } else {
                        self.index += 1;
                        if end == item.index() {
                            self.take(item);
                        } else {
                            self.check_at_index = end;
                        }
                    }
                }
                Stat::Failed => self.base.stat = Stat::Failed,
                _ => {}
            }
        }
        self.base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        let parser = &mut self.inners[self.index];
        match parser.finish(item) {
            Stat::Matched(end) => {
                self.base.add_tokens(parser.take_tokens());
                if self.index == self.len - 1 {
                    self.base.stat = Stat::Matched(end);
                } else if end == item.index() {
                    self.index += 1;
                    self.finish(item);
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

pub fn run<T: PItem>(parsers: Vec<Parser<T>>) -> Parser<T> {
    Parser::Run(Run::new(parsers))
}
