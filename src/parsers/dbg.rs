use super::super::*;

#[derive(Debug)]
pub struct Dbg {
    pub base: BaseParser,
    inner: Box<Parser>,
}
impl Dbg {
    pub fn new(parser: Parser) -> Self {
        Self {
            base: BaseParser::new(),
            inner: Box::new(parser),
        }
    }
}
pub fn dbg(parser: Parser) -> Parser {
    Parser::Dbg(Dbg::new(parser))
}

impl Clone for Dbg {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            inner: self.inner.clone(),
        }
    }
}

impl CharParser for Dbg {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        self.base.stat = self.inner.take_char(ch);
        println!(
            "{}: char={}, byte={}, stat={:?}",
            self.inner.string(),
            ch.value,
            ch.byte,
            self.base.stat
        );
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        self.base.stat = self.inner.finish(ch);
        println!(
            "Finish {}: char={}, byte={}, stat={:?}",
            self.inner.string(),
            ch.value,
            ch.byte,
            self.base.stat
        );
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.inner.reset();
        println!("Reset {}", self.inner.string())
    }

    fn string(&self) -> String {
        format!("Dbg({:?})", self.inner.string())
    }
}
