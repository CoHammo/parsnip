use super::*;

#[derive(Debug, Clone)]
pub struct Branch<T: Parses> {
    ops: Vec<Op<T>>,
    early_commit: bool,
    len: usize,
}

pub fn str<T: Parses>(value: impl ToOps<T>) -> Vec<Op<T>> {
    value.ops()
}

pub fn tok<T: Parses>(value: impl ToOps<T>) -> Vec<Op<T>> {
    let mut ops = value.ops();
    ops.insert(0, Op::StartTok);
    ops.push(Op::EndTok);
    ops
}

pub fn not<T: Parses>(value: impl ToOps<T>) -> Vec<Op<T>> {
    let inner = value.ops();
    let mut ops = vec![Op::Peek(false, inner.len() + 2)];
    ops.extend(inner);
    ops.push(Op::CommitPeek);
    ops
}

pub fn rep<T: Parses>(value: impl ToOps<T>, mut min: u32, mut max: u32) -> Vec<Op<T>> {
    min = match min {
        0 => 1,
        m => m,
    };
    if max > 0 && max <= min {
        max = min;
    }
    let inner = value.ops();
    let len = inner.len();
    let mut ops = vec![Op::StartLoop];
    ops.extend(inner);
    ops.push(Op::EndLoop(len, min, max));
    ops
}

pub fn run<T: Parses>(values: Vec<impl ToOps<T>>) -> Vec<Op<T>> {
    let mut ops = Vec::new();
    for inner in values {
        ops.extend(inner.ops());
    }
    ops
}

pub fn slow_till<T: Parses>(value: impl ToOps<T>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Scope];
    ops.push(Op::Branch(Jmp::Up(3), Jmp::Up(1)));
    ops.push(Op::MatchAny);
    ops.push(Op::Jump(Jmp::Back(2)));
    ops.extend(value.ops());
    ops.push(Op::CommitScope);
    ops
}

pub fn till<T: Parses>(values: impl ToOps<T>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Save];
    ops.extend(values.ops());
    ops.push(Op::PopSave);
    ops
}

pub fn branch<T: Parses>(values: impl ToOps<T>, early_commit: bool) -> Branch<T> {
    let ops = values.ops();
    let len = ops.len() + 1;
    Branch {
        ops,
        early_commit,
        len,
    }
}

pub fn alt<T: Parses>(mut branches: Vec<Branch<T>>) -> Vec<Op<T>> {
    let mut ops = vec![Op::Scope];
    let num_branches = branches.len();

    let mut total_len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i == num_branches - 1 {
            match branch.early_commit {
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

            let add_jump = if branch.early_commit { 2 } else { 1 };
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
