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
pub const PEEK: u8 = 14;
pub const COMMIT_PEEK: u8 = 15;

pub trait ToOps<T: Parses> {
    fn ops(self) -> Vec<Op<T>>;
}

impl ToOps<u8> for &str {
    fn ops(self) -> Vec<Op<u8>> {
        let bytes = self.as_bytes();
        let mut ops = Vec::new();
        for byte in bytes {
            ops.push(Op::Match(*byte))
        }
        ops
    }
}

impl<T: Parses> ToOps<T> for Vec<Op<T>> {
    fn ops(self) -> Vec<Op<T>> {
        self
    }
}

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
    Peek(bool, usize),
    CommitPeek,
    StartTok,
    EndTok,
    Save,
    Unsave,
    StartLoop,
    EndLoop(usize, u32, u32),
}

pub struct MatchArgs<'a> {
    pub slice: &'a [u8],
}
impl<'a> MatchArgs<'a> {
    pub fn size(&self) -> u16 {
        (self.slice.len() + 1) as u16
    }
}

pub struct JumpArgs {
    pub target: u16,
}
impl JumpArgs {
    pub fn size(&self) -> u16 {
        3
    }
}

pub struct BranchArgs {
    pub t1: u16,
    pub t2: u16,
}
impl BranchArgs {
    pub fn size(&self) -> u16 {
        5
    }
}

pub struct PeekArgs {
    pub positive: bool,
    pub target: u16,
}
impl PeekArgs {
    pub fn size(&self) -> u16 {
        4
    }
}

pub struct LoopArgs {
    pub start_ip: u16,
    pub min: u32,
    pub max: u32,
}
impl LoopArgs {
    pub fn size(&self) -> u16 {
        11
    }
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
            Op::Peek(_, _) => PEEK,
            Op::CommitPeek => COMMIT_PEEK,
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
    value_size: u8,
    // len: u16,
    // bytes_len: u16,
}

impl Ops {
    pub fn new<T: Parses>(mut ir: Vec<Op<T>>) -> Self {
        ir.push(Op::Matched);
        if ir.len() >= u16::MAX as usize {
            panic!("Too Many Ops");
        }
        let mut ops: Vec<u8> = Vec::new();
        let mut index_map: Vec<usize> = Vec::new();
        let mut jump_map: Vec<(usize, usize)> = Vec::new();
        // let mut len: u16 = 0;
        for (index, op) in ir.into_iter().enumerate() {
            // len += 1;
            let byte_index = ops.len();
            index_map.push(byte_index);
            ops.push(op.byte());
            match op {
                Op::Match(val) => {
                    let bytes = val.to_bytes();
                    ops.extend(bytes);
                }
                Op::Jump(jump) => {
                    match jump {
                        Jmp::Up(add) => jump_map.push((byte_index, index + add)),
                        Jmp::Back(sub) => jump_map.push((byte_index, index - sub)),
                    }
                    ops.extend([0, 0]);
                }
                Op::Peek(positive, len) => {
                    ops.push(positive as u8);
                    jump_map.push((byte_index + 1, index + len));
                    ops.extend([0, 0]);
                }
                Op::Branch(j1, j2) => {
                    match j1 {
                        Jmp::Up(add) => jump_map.push((byte_index, index + add)),
                        Jmp::Back(sub) => jump_map.push((byte_index, index - sub)),
                    }
                    match j2 {
                        Jmp::Up(add) => jump_map.push((byte_index + 2, index + add)),
                        Jmp::Back(sub) => jump_map.push((byte_index + 2, index - sub)),
                    }
                    ops.extend([0, 0, 0, 0]);
                }
                Op::EndLoop(jump_back, min, max) => {
                    jump_map.push((byte_index, index - jump_back));
                    ops.extend([0, 0]);
                    ops.extend(min.to_be_bytes());
                    ops.extend(max.to_be_bytes());
                }
                _ => {}
            }
        }
        for (from_byte_index, target_ir_index) in jump_map {
            let target_byte_index = index_map[target_ir_index];
            let upper = (target_byte_index >> 8) as u8;
            let lower = target_byte_index as u8;
            ops[from_byte_index + 1] = upper;
            ops[from_byte_index + 2] = lower;
        }
        // let bytes_len = ops.len() as u16;
        Self {
            ops,
            value_size: std::mem::size_of::<T>() as u8,
            // len,
            // bytes_len,
        }
    }

    // pub fn op_len(&self) -> u16 {
    //     self.len
    // }

    // pub fn bytes_len(&self) -> u16 {
    //     self.bytes_len
    // }

    pub fn get_match_args(&self, index: u16) -> MatchArgs<'_> {
        let slice = unsafe {
            &self.ops.get_unchecked(
                (index + 1) as usize..((index + 1 + self.value_size as u16) as usize),
            )
        };
        MatchArgs { slice }
    }

    pub fn get_jump_args(&self, index: u16) -> JumpArgs {
        let target = u16::from_be_bytes([self[index + 1], self[index + 2]]);
        JumpArgs { target }
    }

    pub fn get_peek_args(&self, index: u16) -> PeekArgs {
        let positive = self[index + 1] != 0;
        let target = u16::from_be_bytes([self[index + 2], self[index + 3]]);
        PeekArgs { positive, target }
    }

    pub fn get_branch_args(&self, index: u16) -> BranchArgs {
        let t1 = u16::from_be_bytes([self[index + 1], self[index + 2]]);
        let t2 = u16::from_be_bytes([self[index + 3], self[index + 4]]);
        BranchArgs { t1, t2 }
    }

    pub fn get_loop_args(&self, index: u16) -> LoopArgs {
        let start_ip = u16::from_be_bytes([self[index + 1], self[index + 2]]);
        let min = u32::from_be_bytes([
            self[index + 3],
            self[index + 4],
            self[index + 5],
            self[index + 6],
        ]);
        let max = u32::from_be_bytes([
            self[index + 7],
            self[index + 8],
            self[index + 9],
            self[index + 10],
        ]);
        LoopArgs { start_ip, min, max }
    }

    pub fn get_info_at(&self, index: u16) -> (u8, String, u16) {
        match self[index] {
            MATCHED => (MATCHED, format!("{}:Matched", index), 1),
            MATCH => {
                let args = self.get_match_args(index);
                (
                    MATCH,
                    format!("{}:Match({:?})", index, args.slice),
                    args.size(),
                )
            }
            MATCH_ANY => (MATCH_ANY, format!("{}:MatchAny", index), 1),
            JUMP => {
                let args = self.get_jump_args(index);
                (
                    JUMP,
                    format!("{}:Jump({})", index, args.target),
                    args.size(),
                )
            }
            BRANCH => {
                let args = self.get_branch_args(index);
                (
                    BRANCH,
                    format!("{}:Branch({}, {})", index, args.t1, args.t2),
                    args.size(),
                )
            }
            SCOPE => (SCOPE, format!("{}:Scope", index), 1),
            COMMIT_SCOPE => (COMMIT_SCOPE, format!("{}:CommitScope", index), 1),
            KILL_SCOPE => (KILL_SCOPE, format!("{}:KillScope", index), 1),
            PEEK => {
                let args = self.get_peek_args(index);
                (
                    PEEK,
                    format!("{}:Peek({}, {})", index, args.positive, args.target),
                    args.size(),
                )
            }
            COMMIT_PEEK => (COMMIT_PEEK, format!("{}:CommitPeek", index), 1),
            START_TOK => (START_TOK, format!("{}:StartTok", index), 1),
            END_TOK => (END_TOK, format!("{}:EndTok", index), 1),
            SAVE => (SAVE, format!("{}:Save", index), 1),
            UNSAVE => (UNSAVE, format!("{}:Unsave", index), 1),
            START_LOOP => (START_LOOP, format!("{}:StartLoop", index), 1),
            END_LOOP => {
                let args = self.get_loop_args(index);
                (
                    END_LOOP,
                    format!(
                        "{}:EndLoop({}, {}, {})",
                        index, args.start_ip, args.min, args.max
                    ),
                    args.size(),
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
    fn index(&self, index: u16) -> &Self::Output {
        unsafe { self.ops.get_unchecked(index as usize) }
    }
}
