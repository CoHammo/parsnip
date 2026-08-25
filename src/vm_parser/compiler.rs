use super::*;

#[derive(Debug, Clone)]
pub struct Branch<T: Matches> {
    comms: Vec<Comm<T>>,
    commits: bool,
    len: usize,
}

impl<T: Matches> ToComms<T> for Vec<Comm<T>> {
    fn to_comms(self) -> Vec<Comm<T>> {
        self
    }
}

pub fn str<T: Matches>(value: impl ToComms<T>) -> Vec<Comm<T>> {
    value.to_comms()
}

pub fn tok<T: Matches>(value: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = value.to_comms();
    comms.insert(0, Comm::Tok(true));
    comms.push(Comm::Tok(false));
    comms
}

pub fn not<T: Matches>(value: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Scope];
    let not = value.to_comms();
    comms.push(Comm::Branch(true, not.len() + 2, true, 1));
    comms.extend(not);
    comms.push(Comm::KillScope);
    comms
}

pub fn rep<T: Matches>(value: impl ToComms<T>, mut min: usize, mut max: usize) -> Vec<Comm<T>> {
    min = match min {
        0 => 1,
        m => m,
    };
    if max > 0 && max <= min {
        max = min;
    }
    let inner = value.to_comms();
    let mut comms = vec![Comm::StartLoop];
    comms.extend(inner);
    comms.push(Comm::EndLoop(min, max));
    comms
}

pub fn run<T: Matches>(values: Vec<impl ToComms<T>>) -> Vec<Comm<T>> {
    let mut all = Vec::new();
    for value in values {
        all.extend(value.to_comms());
    }
    all
}

pub fn till2<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Scope];
    let value = values.to_comms();
    comms.push(Comm::Branch(true, 3, true, 1));
    comms.extend(vec![Comm::MatchAny, Comm::Jump(false, 2)]);
    comms.extend(value);
    comms.push(Comm::CommitScope);
    comms
}

pub fn till<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Save];
    comms.extend(values.to_comms());
    comms.push(Comm::Unsave);
    comms
}

pub fn branch<T: Matches>(values: impl ToComms<T>, commits: bool) -> Branch<T> {
    let comms = values.to_comms();
    let len = comms.len() + 1;
    Branch {
        comms,
        commits,
        len,
    }
}

pub fn alt<T: Matches>(mut branches: Vec<Branch<T>>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Scope];
    let num_branches = branches.len();

    let mut total_len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i != num_branches - 1 {
            total_len += branch.len;
        } else {
            match branch.commits {
                true => {
                    total_len += branch.len;
                    branch.comms.push(Comm::Jump(true, 2));
                }
                false => total_len += branch.len - 1,
            }
        }
    }

    let mut len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i != num_branches - 1 {
            let branches_left = num_branches - 2 - i;
            len += branch.len;
            comms.push(Comm::Branch(true, 1, true, len + branches_left + 1));

            let add = if branch.commits { 2 } else { 1 };
            branch.comms.push(Comm::Jump(true, total_len - len + add));
        }
    }

    for branch in branches {
        comms.extend(branch.comms);
    }
    comms.push(Comm::CommitScope);

    comms
}

pub fn commit<T: Matches>() -> Vec<Comm<T>> {
    vec![Comm::CommitScope]
}
