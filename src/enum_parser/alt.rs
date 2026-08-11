// use super::super::types::*;
use super::*;
use crate::enum_parser::Parser;

#[derive(Debug)]
pub struct Alt<T: PItem> {
    pub base: BaseParser,
    inners: Box<[(bool, Parser<T>)]>,
}
impl<T: PItem> Alt<T> {
    pub fn new(parsers: Vec<Parser<T>>) -> Self {
        Self {
            base: BaseParser::new(),
            inners: parsers.into_iter().map(|p| (true, p)).collect(),
        }
    }
}
pub fn alt<T: PItem>(parsers: Vec<Parser<T>>) -> Parser<T> {
    Parser::Alt(Alt::new(parsers))
}

impl<T: PItem> Clone for Alt<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inners: self.inners.iter().map(|(_, p)| (true, p.clone())).collect(),
        }
    }
}

impl<T: PItem> ItemParser<T> for Alt<T> {
    fn take<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        freshen!(self, item);
        let mut running = false;
        for parser in &mut self.inners {
            if parser.0 {
                running = true;
                match parser.1.take(item) {
                    Stat::Running => {}
                    Stat::Matched(byte) => {
                        self.base.add_tokens(parser.1.take_tokens());
                        self.base.stat = Stat::Matched(byte);
                        break;
                    }
                    Stat::Failed => {
                        parser.0 = false;
                    }
                }
            }
        }
        if !running {
            self.base.stat = Stat::Failed;
        }
        self.base.stat
    }

    fn finish<I: Iterator<Item = T> + Clone>(&mut self, item: &ParseItem<T, I>) -> Stat {
        for parser in &mut self.inners {
            if parser.0 {
                match parser.1.finish(item) {
                    Stat::Matched(byte) => {
                        self.base.add_tokens(parser.1.take_tokens());
                        self.base.stat = Stat::Matched(byte);
                        break;
                    }
                    _ => {}
                }
            }
        }
        if let Stat::Running = self.base.stat {
            self.base.stat = Stat::Failed;
        }
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            for parser in &mut self.inners {
                parser.0 = true;
                parser.1.reset();
            }
        }
    }

    fn string(&self) -> String {
        format!(
            "Alt([{}])",
            self.inners
                .iter()
                .map(|p| p.1.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
