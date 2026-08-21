use super::*;

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

// pub fn till<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
//     let mut comms = vec![
//         Comm::Scope,
//         Comm::Branch(Jump::Up(1), Jump::Up(3)),
//         Comm::MatchAny,
//         Comm::Jump(Jump::Back(2)),
//     ];
//     comms.extend(values.to_comms());
//     comms.push(Comm::Commit);
//     comms
// }

pub fn till<T: Matches>(values: impl ToComms<T>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Save];
    comms.extend(values.to_comms());
    comms.push(Comm::Unsave);
    comms
}

pub fn alt<T: Matches>(values: Vec<impl ToComms<T>>) -> Vec<Comm<T>> {
    let mut comms = vec![Comm::Scope];
    let mut branches: Vec<Vec<Comm<T>>> = Vec::new();
    let num_branches: usize = values.len();

    let mut total_len: usize = 0;
    for (i, value) in values.into_iter().enumerate() {
        let branch = value.to_comms();
        if i == num_branches - 1 {
            total_len += branch.len();
        } else {
            total_len += branch.len() + 1;
        }
        branches.push(branch);
    }

    let mut num_branches_left: usize = num_branches - 2;
    let mut len: usize = 0;
    for (i, branch) in branches.iter_mut().enumerate() {
        if i != num_branches - 1 {
            len += branch.len() + 1;
            comms.push(Comm::Branch(true, 1, true, len + num_branches_left + 1));
            num_branches_left -= 1;

            total_len -= branch.len() + 1;
            branch.push(Comm::Jump(true, total_len + 1));
        }
    }

    for branch in branches {
        comms.extend(branch);
    }

    comms.push(Comm::CommitScope);
    comms
}
