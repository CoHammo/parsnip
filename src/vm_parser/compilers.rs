use super::*;

pub trait Compiles<T: Parses> {
    fn cops(self) -> Vec<Op<T>>;
}

impl Compiles<u8> for &str {
    fn cops(self) -> Vec<Op<u8>> {
        let bytes = self.as_bytes();
        let mut ops = Vec::new();
        for byte in bytes {
            ops.push(Op::Match(*byte))
        }
        ops
    }
}

impl<T: Parses> Compiles<T> for Vec<Op<T>> {
    fn cops(self) -> Vec<Op<T>> {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Branch<T: Parses> {
    ops: Vec<Op<T>>,
    commits: bool,
    len: usize,
}

pub fn str<T: Parses>(value: impl Compiles<T>) -> Vec<Op<T>> {
    value.cops()
}

pub fn tok<T: Parses>(value: impl Compiles<T>) -> Vec<Op<T>> {
    let mut ops = value.cops();
    ops.insert(0, Op::StartTok);
    ops.push(Op::EndTok);
    ops
}

pub fn not<T: Parses>(value: impl Compiles<T>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Scope];
    let inner = value.cops();
    let len = inner.len() + 2;
    ops.push(Op::Branch(Jmp::Up(len), Jmp::Up(1)));
    ops.extend(inner);
    ops.push(Op::KillScope);
    ops
}

pub fn rep<T: Parses>(value: impl Compiles<T>, mut min: u32, mut max: u32) -> Vec<Op<T>> {
    min = match min {
        0 => 1,
        m => m,
    };
    if max > 0 && max <= min {
        max = min;
    }
    let inner = value.cops();
    let len = inner.len();
    let mut ops = vec![Op::StartLoop];
    ops.extend(inner);
    ops.push(Op::EndLoop(len, min, max));
    ops
}

pub fn run<T: Parses>(values: Vec<impl Compiles<T>>) -> Vec<Op<T>> {
    let mut ops = Vec::new();
    for inner in values {
        ops.extend(inner.cops());
    }
    ops
}

pub fn till2<T: Parses>(value: impl Compiles<T>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Scope];
    ops.push(Op::Branch(Jmp::Up(3), Jmp::Up(1)));
    ops.push(Op::MatchAny);
    ops.push(Op::Jump(Jmp::Back(2)));
    ops.extend(value.cops());
    ops.push(Op::CommitScope);
    ops
}

pub fn till<T: Parses>(values: impl Compiles<T>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Save];
    ops.extend(values.cops());
    ops.push(Op::Unsave);
    ops
}

pub fn branch<T: Parses>(values: impl Compiles<T>, commits: bool) -> Branch<T> {
    let ops = values.cops();
    let len = ops.len() + 1;
    Branch { ops, commits, len }
}

pub fn alt<T: Parses>(mut branches: Vec<Branch<T>>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Scope];
    let num_branches = branches.len();

    let mut total_len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i == num_branches - 1 {
            match branch.commits {
                true => {
                    total_len += branch.len;
                    branch.ops.push(Op::Jump(Jmp::Up(2)));
                }
                false => total_len += branch.len - 1,
            }
        } else {
            total_len += branch.len;
        }
    }

    let mut len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i != num_branches - 1 {
            let branch_ops_left = num_branches - 2 - i;
            len += branch.len;
            ops.push(Op::Branch(Jmp::Up(1), Jmp::Up(len + branch_ops_left + 1)));

            let add_jump = if branch.commits { 2 } else { 1 };
            branch
                .ops
                .push(Op::Jump(Jmp::Up(total_len - len + add_jump)));
        }
    }

    for branch in branches {
        ops.extend(branch.ops);
    }
    ops.push(Op::CommitScope);
    ops
}

pub fn commit<T: Parses>() -> Vec<Op<T>> {
    vec![Op::CommitScope]
}
