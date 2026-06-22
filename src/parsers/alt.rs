use super::super::*;

#[derive(Debug)]
pub struct Alt {
    pub base: BaseParser,
    inners: Box<[(bool, Parser)]>,
}
impl Alt {
    pub fn new(parsers: Vec<Parser>) -> Self {
        Self {
            base: BaseParser::new(),
            inners: parsers.into_iter().map(|p| (true, p)).collect(),
        }
    }
}
pub fn alt(parsers: Vec<Parser>) -> Parser {
    Parser::Alt(Alt::new(parsers))
}

impl Clone for Alt {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inners: self.inners.iter().map(|(_, p)| (true, p.clone())).collect(),
        }
    }
}

impl CharParser for Alt {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        let mut running = false;
        for parser in &mut self.inners {
            if parser.0 {
                running = true;
                match parser.1.take_char(ch) {
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

    fn finish(&mut self, ch: &Char) -> Stat {
        for parser in &mut self.inners {
            if parser.0 {
                match parser.1.finish(ch) {
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
