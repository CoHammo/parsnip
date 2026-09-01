use super::Parses;
use std::ops::Index;

pub const MATCHED: u8 = 0;
pub const MATCH: u8 = 1;
pub const MATCH_ANY: u8 = 2;
pub const JUMP: u8 = 3;
pub const BRANCH: u8 = 4;
pub const SCOPE: u8 = 5;
pub const COMMIT_SCOPE: u8 = 6;
pub const KILL_SCOPE: u8 = 7;
pub const SAVE: u8 = 8;
pub const UNSAVE: u8 = 9;
pub const START_LOOP: u8 = 10;
pub const END_LOOP: u8 = 11;
pub const START_TOK: u8 = 12;
pub const END_TOK: u8 = 13;

#[derive(Debug, Clone)]
pub enum Jmp {
    Up(usize),
    Back(usize),
}

#[derive(Debug, Clone)]
pub enum Op<T: Parses> {
    Matched,
    Match(T),
    MatchAny,
    Jump(Jmp),
    Branch(Jmp, Jmp),
    Scope,
    CommitScope,
    KillScope,
    StartTok,
    EndTok,
    Save,
    Unsave,
    StartLoop,
    EndLoop(usize, u32, u32),
}

impl<T: Parses> Op<T> {
    pub fn byte(&self) -> u8 {
        match self {
            Op::Matched => MATCHED,
            Op::Match(_) => MATCH,
            Op::MatchAny => MATCH_ANY,
            Op::Jump(_) => JUMP,
            Op::Branch(_, _) => BRANCH,
            Op::Scope => SCOPE,
            Op::CommitScope => COMMIT_SCOPE,
            Op::KillScope => KILL_SCOPE,
            Op::StartTok => START_TOK,
            Op::EndTok => END_TOK,
            Op::Save => SAVE,
            Op::Unsave => UNSAVE,
            Op::StartLoop => START_LOOP,
            Op::EndLoop(_, _, _) => END_LOOP,
        }
    }
}

#[derive(Debug)]
pub struct Ops {
    ops: Vec<u8>,
    data_len: u8,
}

impl Ops {
    pub fn new<T: Parses>(ir: Vec<Op<T>>) -> Self {
        if ir.len() >= u16::MAX as usize {
            panic!("Too Many Ops");
        }
        let mut ops: Vec<u8> = Vec::new();
        let mut indices: Vec<usize> = Vec::new();
        let mut jumps: Vec<(usize, usize)> = Vec::new();
        for (i, op) in ir.into_iter().enumerate() {
            let index = i;
            let byte_index = ops.len();
            indices.push(byte_index);
            ops.push(op.byte());
            match op {
                Op::Match(val) => {
                    let bytes = val.to_bytes();
                    ops.extend(bytes);
                }
                Op::Jump(jump) => {
                    match jump {
                        Jmp::Up(add) => jumps.push((byte_index, index + add)),
                        Jmp::Back(sub) => jumps.push((byte_index, index - sub)),
                    }
                    ops.extend([0, 0]);
                }
                Op::Branch(j1, j2) => {
                    match j1 {
                        Jmp::Up(add) => jumps.push((byte_index, index + add)),
                        Jmp::Back(sub) => jumps.push((byte_index, index - sub)),
                    }
                    match j2 {
                        Jmp::Up(add) => jumps.push((byte_index + 2, index + add)),
                        Jmp::Back(sub) => jumps.push((byte_index + 2, index - sub)),
                    }
                    ops.extend([0, 0, 0, 0]);
                }
                Op::EndLoop(jump_back, min, max) => {
                    jumps.push((byte_index, index - jump_back));
                    ops.extend([0, 0]);
                    ops.extend(min.to_be_bytes());
                    ops.extend(max.to_be_bytes());
                }
                _ => {}
            }
        }
        for (at, to) in jumps {
            let target_index = indices[to];
            let up = (target_index >> 8) as u8;
            let low = target_index as u8;
            ops[at + 1] = up;
            ops[at + 2] = low;
        }
        ops.push(MATCHED);

        Self {
            ops,
            data_len: T::bytes_len(),
        }
    }

    pub fn get_match_slice(&self, index: u16) -> &[u8] {
        unsafe {
            &self
                .ops
                .get_unchecked((index + 1) as usize..((index + 1 + self.data_len as u16) as usize))
        }
    }

    pub fn get_jump_target(&self, index: u16) -> u16 {
        let up = self[index + 1];
        let low = self[index + 2];
        (up as u16) << 8 | low as u16
    }

    pub fn get_branch_targets(&self, index: u16) -> (u16, u16) {
        let up1 = self[index + 1];
        let low1 = self[index + 2];
        let up2 = self[index + 3];
        let low2 = self[index + 4];
        (
            (up1 as u16) << 8 | low1 as u16,
            (up2 as u16) << 8 | low2 as u16,
        )
    }

    pub fn get_loop_bounds(&self, index: u16) -> (u16, u32, u32) {
        let startupper = self[index + 1];
        let startlower = self[index + 2];
        let start = (startupper as u16) << 8 | startlower as u16;

        let up1 = self[index + 3];
        let upmid1 = self[index + 4];
        let lowmid1 = self[index + 5];
        let low1 = self[index + 6];
        let min = (up1 as u32) << 24 | (upmid1 as u32) << 16 | (lowmid1 as u32) << 8 | low1 as u32;

        let up2 = self[index + 7];
        let upmid2 = self[index + 8];
        let lowmid2 = self[index + 9];
        let low2 = self[index + 10];
        let max = (up2 as u32) << 24 | (upmid2 as u32) << 16 | (lowmid2 as u32) << 8 | low2 as u32;
        (start, min, max)
    }

    pub fn get_info_at(&self, index: u16) -> (u8, String, u8) {
        match self[index] {
            MATCHED => (MATCHED, format!("{}:Matched", index), 1),
            MATCH => {
                let thing = self.get_match_slice(index);
                (
                    MATCH,
                    format!("{}:Match({:?})", index, thing),
                    (thing.len() + 2) as u8,
                )
            }
            MATCH_ANY => (MATCH_ANY, format!("{}:MatchAny", index), 1),
            JUMP => {
                let target = self.get_jump_target(index);
                (JUMP, format!("{}:Jump({})", index, target), 3)
            }
            BRANCH => {
                let (target1, target2) = self.get_branch_targets(index);
                (
                    BRANCH,
                    format!("{}:Branch({}, {})", index, target1, target2),
                    5,
                )
            }
            SCOPE => (SCOPE, format!("{}:Scope", index), 1),
            COMMIT_SCOPE => (COMMIT_SCOPE, format!("{}:CommitScope", index), 1),
            KILL_SCOPE => (KILL_SCOPE, format!("{}:KillScope", index), 1),
            START_TOK => (START_TOK, format!("{}:StartTok", index), 1),
            END_TOK => (END_TOK, format!("{}:EndTok", index), 1),
            SAVE => (SAVE, format!("{}:Save", index), 1),
            UNSAVE => (UNSAVE, format!("{}:Unsave", index), 1),
            START_LOOP => (START_LOOP, format!("{}:StartLoop", index), 1),
            END_LOOP => {
                let (start, min, max) = self.get_loop_bounds(index);
                (
                    END_LOOP,
                    format!("{}:EndLoop({}, {}, {})", index, start, min, max),
                    9,
                )
            }
            op => {
                panic!("Bad Op Code {}", op)
            }
        }
    }

    pub fn debug_str(&self, pretty: bool) -> String {
        let mut s = String::new();
        let mut index = 0;
        let mut count = 0;
        loop {
            let (code, op_str, len) = self.get_info_at(index);
            s.push_str(&format!("({}){}, ", count, op_str));
            if pretty {
                s.push('\n');
            }
            index += len as u16;
            count += 1;
            if code == MATCHED {
                break;
            }
        }
        s
    }
}

impl Index<u16> for Ops {
    type Output = u8;
    fn index(&self, index: u16) -> &u8 {
        unsafe { self.ops.get_unchecked(index as usize) }
    }
}
