use super::super::*;

pub struct Dbg {
    pub base: BaseParser,
    _inner: Box<Parser>,
}
impl Dbg {
    pub fn new(parser: Parser) -> Self {
        Self {
            base: BaseParser::new(),
            _inner: Box::new(parser),
        }
    }
}
