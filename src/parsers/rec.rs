use super::super::*;

#[derive(Debug, Clone)]
pub enum RecState {
    Before,
    Inner(Box<Parser>),
    After,
    Done,
}

parser!(Rec rec {
    state: RecState = RecState::Before,
    before: Parser => bef: Box<Parser>,
    base_parser: Parser => base: Box<Parser>,
    after: Parser => aft: Box<Parser>,
    run_base: bool = true,
    inner_tokens: Option<Vec<Token>> = None,
} {
    bef = Box::new(before);
    base = Box::new(base_parser);
    aft = Box::new(after);
});

impl Clone for Rec {
    fn clone(&self) -> Self {
        Rec::new(*self.bef.clone(), *self.base.clone(), *self.aft.clone())
    }
}

impl CharParser for Rec {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        match &mut self.state {
            RecState::Before => match self.bef.take_char(ch) {
                Stat::Running | Stat::HasMatch(_) => {}
                Stat::Matched(_) => {
                    self.state = RecState::Inner(Box::new(Parser::Rec(self.clone())))
                }
                Stat::Failed => self.state = RecState::Done,
            },
            RecState::Inner(inner) => match inner.take_char(ch) {
                Stat::Running | Stat::HasMatch(_) => {}
                Stat::Matched(_) => {
                    self.inner_tokens = inner.take_tokens();
                    self.state = RecState::After;
                }
                Stat::Failed => self.state = RecState::Done,
            },
            RecState::After => match self.aft.take_char(ch) {
                Stat::Running => {}
                Stat::HasMatch(end_byte) => {
                    self.run_base = false;
                    self.stat = Stat::HasMatch(end_byte);
                }
                Stat::Matched(end_byte) => {
                    let betoks = self.bef.take_tokens();
                    let inner_toks = self.inner_tokens.take();
                    let atoks = self.aft.take_tokens();
                    self.add_tokens(betoks);
                    self.add_tokens(inner_toks);
                    self.add_tokens(atoks);
                    self.run_base = false;
                    self.state = RecState::Done;
                    self.stat = Stat::Matched(end_byte);
                }
                Stat::Failed => {
                    self.state = RecState::Done;
                    if !self.run_base {
                        self.stat = Stat::Failed;
                    }
                }
            },
            RecState::Done => {}
        }
        if self.run_base {
            match self.base.take_char(ch) {
                Stat::Running => {}
                Stat::HasMatch(end_byte) => {
                    self.state = RecState::Done;
                    self.stat = Stat::HasMatch(end_byte);
                }
                Stat::Matched(end_byte) => {
                    let toks = self.base.take_tokens();
                    self.add_tokens(toks);
                    self.state = RecState::Done;
                    self.stat = Stat::Matched(end_byte);
                }
                Stat::Failed => {
                    self.run_base = false;
                    if let RecState::Done = self.state {
                        self.stat = Stat::Failed;
                    }
                }
            }
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if let RecState::Inner(inner) = &mut self.state {
            match inner.finish(ch) {
                Stat::Matched(_) => {
                    self.inner_tokens = inner.take_tokens();
                    self.state = RecState::After;
                }
                _ => {
                    self.state = RecState::Done;
                    if !self.run_base {
                        self.stat = Stat::Failed;
                    }
                }
            }
        }
        if let RecState::After = self.state {
            match self.aft.finish(ch) {
                Stat::Matched(end_byte) => {
                    let betoks = self.bef.take_tokens();
                    let inner_toks = self.inner_tokens.take();
                    let atoks = self.aft.take_tokens();
                    self.add_tokens(betoks);
                    self.add_tokens(inner_toks);
                    self.add_tokens(atoks);
                    self.state = RecState::Done;
                    self.stat = Stat::Matched(end_byte);
                    self.run_base = false;
                }
                _ => {
                    self.state = RecState::Done;
                    if !self.run_base {
                        self.stat = Stat::Failed;
                    }
                }
            }
        }
        if self.run_base {
            match self.base.finish(ch) {
                Stat::Matched(end_byte) => {
                    let toks = self.base.take_tokens();
                    self.add_tokens(toks);
                    self.state = RecState::Done;
                    self.stat = Stat::Matched(end_byte);
                }
                _ => {
                    self.state = RecState::Done;
                    self.stat = Stat::Failed;
                }
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.state = RecState::Before;
        self.bef.reset();
        self.base.reset();
        self.aft.reset();
        self.run_base = true;
        self.inner_tokens = None;
    }

    fn string(&self) -> String {
        format!(
            "Rec({} ... {} | {})",
            self.bef.string(),
            self.aft.string(),
            self.base.string()
        )
    }
}
