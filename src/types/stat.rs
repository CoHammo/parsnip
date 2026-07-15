#[derive(Debug, Clone, Copy)]
pub enum Stat {
    Running,
    // PossibleMatch(usize),
    Matched(usize),
    Failed,
}
