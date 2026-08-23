use super::iter::Matches;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    Running,
    Matched,
    Failed,
}

#[derive(Debug, Clone)]
pub enum Comm<T: Matches> {
    Matched,
    Match(T),
    MatchAny,
    Jump(bool, usize),
    Branch(bool, usize, bool, usize),
    Scope,
    CommitScope,
    KillScope,
    Tok(bool),
    Save,
    Unsave,
    StartLoop,
    EndLoop(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Loop(Loop),
    // Call(usize),
    Scope(usize),
    Save { ip: usize, event: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loop {
    pub start: usize,
    pub count: usize,
}
impl Loop {
    pub fn new(start: usize) -> Self {
        Self { start, count: 0 }
    }
}
