use super::{Jmp, Op, Parser, Parses, Snip, Stat, Var, ops::*};
use std::ops::Index;

pub type OpFunc<T> =
    fn(vm: &mut Parser<T>, snip: &Snip<T>, tid: u16, args: Args, next1: &Fop<T>, next2: &Fop<T>);

#[derive(Debug)]
pub struct Args<'a> {
    data: &'a [u8],
    id: u16,
}

impl<'a> Args<'a> {
    pub fn new(data: &'a [u8], id: u16) -> Self {
        Self { data, id }
    }

    pub fn at(self, id: u16) -> Self {
        Self { id, ..self }
    }

    pub fn get_match_args(&self, len: u16) -> &[u8] {
        let slice = unsafe {
            &self
                .data
                .get_unchecked((self.id) as usize..((self.id + len) as usize))
        };
        slice
    }

    // pub fn get_jump_args(&self) -> (u16, OpFunc<T>) {
    //     let args_target = u16::from_be_bytes([self[self.id], self[self.id + 1]]);
    //     let func = self[self[self.id + 2]];
    //     (args_target, func)
    // }

    // pub fn get_branch_args(&self) -> ((u16, OpFunc<T>), (u16, OpFunc<T>)) {
    //     let args_target1 = u16::from_be_bytes([self[self.id], self[self.id + 1]]);
    //     let args_target2 = u16::from_be_bytes([self[self.id + 2], self[self.id + 3]]);
    //     (
    //         (args_target1, self[self[self.id + 4]]),
    //         (args_target2, self[self[self.id + 5]]),
    //     )
    // }

    pub fn get_loop_args(&self) -> (u32, u32) {
        let min = u32::from_be_bytes([
            self[self.id],
            self[self.id + 1],
            self[self.id + 2],
            self[self.id + 3],
        ]);
        let max = u32::from_be_bytes([
            self[self.id + 4],
            self[self.id + 5],
            self[self.id + 6],
            self[self.id + 7],
        ]);
        (min, max)
    }
}

impl Index<u16> for Args<'_> {
    type Output = u8;
    fn index(&self, index: u16) -> &u8 {
        unsafe { self.data.get_unchecked(index as usize) }
    }
}

pub fn nothing<T: Parses>(_: &mut Parser<T>, _: &Snip<T>, _: u16, _: Args, _: &Fop<T>, _: &Fop<T>) {
    panic!("Nothing Fop Called!!");
}

pub fn matched<T: Parses>(
    vm: &mut Parser<T>,
    _: &Snip<T>,
    tid: u16,
    _: Args,
    _: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    if let Some(best) = vm.best_match
        && best != 0
    {
        vm.events.unref(best);
    }
    vm.best_match = Some(thread.event);
    vm.kill_thread(tid, false);
}

pub fn try_match<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    if let Some(value) = &snip.value {
        let slice = args.get_match_args(T::bytes_len() as u16);
        if value.matches(slice) {
            let thread = &mut vm.threads[tid];
            thread.ip = fop1;
            // } else if thread.saves > 0 {
            //     if self.debug {
            //         println!("    Rewinding...");
            //     }
            // let event = thread.event;
            // let scope = thread.scope.val();
            // thread.rewind(&mut self.state);
            // self.scopes.kill_scopes(thread.scope.diff(scope));
            // self.events.upref(thread.event);
            // self.events.unref(event);
        } else {
            vm.kill_thread(tid, true);
        }
    } else {
        vm.kill_thread(tid, true);
    }
}

pub fn match_any<T: Parses>(
    vm: &mut Parser<T>,
    _: &Snip<T>,
    tid: u16,
    _: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    thread.ip = fop1;
}

pub fn jump<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    fop1.call(vm, snip, tid, args);
}

pub fn branch_off<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    fop2: &Fop<T>,
) {
    let fork = vm.threads.fork_thread(tid);
    fork.ip = fop2;
    vm.stack.upref(fork.stack);
    vm.events.upref(fork.event);
    fop1.call(vm, snip, tid, args);
}

pub fn scope<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    vm.threads[tid].scope.add_scope(vm.scopes.next_scope());
    fop1.call(vm, snip, tid, args);
}

pub fn commit_scope<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    if let Some(scope_id) = vm.threads[tid].scope.pop_scope() {
        vm.scopes.kill_scope(scope_id);
        fop1.call(vm, snip, tid, args);
    } else {
        println!("Tried to commit a scope that doesn't exist");
        vm.stat = Stat::Failed;
    }
}

pub fn kill_scope<T: Parses>(
    vm: &mut Parser<T>,
    _: &Snip<T>,
    tid: u16,
    _: Args,
    _: &Fop<T>,
    _: &Fop<T>,
) {
    if let Some(scope_id) = vm.threads[tid].scope.last_scope() {
        vm.scopes.kill_scope(scope_id);
    } else {
        println!("Tried to kill a scope that doesn't exist");
        vm.stat = Stat::Failed;
    }
}

pub fn save<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    thread.ip = fop1;
    thread.stack = vm.stack.push_stack(
        Var::save(thread.ip, thread.event, thread.scope),
        thread.stack,
    );
    thread.saves += 1;
    fop1.call(vm, snip, tid, args);
}

pub fn unsave<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    if let Some((prev, Var::Save { .. })) = vm.stack.pop_stack(thread.stack) {
        thread.stack = prev;
        thread.saves -= 1;
        fop1.call(vm, snip, tid, args);
    } else {
        println!("Tried to unsave without a save");
        vm.stat = Stat::Failed;
    }
}

pub fn start_tok<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    thread.event = vm.events.push_event(true, snip.index, thread.event);
    fop1.call(vm, snip, tid, args);
}

pub fn end_tok<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    thread.event = vm.events.push_event(false, snip.index, thread.event);
    fop1.call(vm, snip, tid, args);
}

pub fn start_loop<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    _: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    thread.stack = vm.stack.push_stack(Var::loo(), thread.stack);
    fop1.call(vm, snip, tid, args);
}

pub fn end_loop<T: Parses>(
    vm: &mut Parser<T>,
    snip: &Snip<T>,
    tid: u16,
    args: Args,
    fop1: &Fop<T>,
    fop2: &Fop<T>,
) {
    let thread = &mut vm.threads[tid];
    if let Some((new_stack_id, Var::Loop(loo))) = vm.stack.edit(thread.stack) {
        thread.stack = new_stack_id;
        loo.count += 1;
        let (min, max) = args.get_loop_args();
        // println!("Loop Bounds at {}: {}, {}", args_id, min, max);
        if loo.count == max {
            thread.stack = vm.stack.pop_stack(thread.stack).unwrap().0;
            fop2.call(vm, snip, tid, args);
        } else {
            if loo.count >= min {
                let fork = vm.threads.fork_thread(tid);
                fork.ip = fop2;
                fork.stack = vm.stack.before(fork.stack);
                vm.stack.upref(fork.stack);
                vm.events.upref(fork.event);
            }
            fop1.call(vm, snip, tid, args);
        }
    } else {
        println!("Tried to close a loop with no start");
        vm.stat = Stat::Failed;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fop<T: Parses> {
    pub fop: OpFunc<T>,
    pub fop_id: u8,
    pub args_id: u16,
    pub next1: *const Fop<T>,
    pub next2: *const Fop<T>,
}

impl<T: Parses> Fop<T> {
    pub fn new(fop: OpFunc<T>, fop_id: u8, args_id: u16) -> Self {
        Self {
            fop,
            fop_id,
            args_id,
            next1: 0 as *const Fop<T>,
            next2: 0 as *const Fop<T>,
        }
    }

    pub fn call(&self, vm: &mut Parser<T>, snip: &Snip<T>, tid: u16, args: Args) {
        unsafe {
            (self.fop)(
                vm,
                snip,
                tid,
                args.at(self.args_id),
                &*self.next1,
                &*self.next2,
            )
        }
    }
}

#[derive(Debug)]
pub struct IRFop {
    pub fop_id: u8,
    pub next_index1: usize,
    pub next_index2: usize,
    // pub name: String,
}

impl IRFop {
    pub fn new(
        fop_id: u8,
        next_index1: usize,
        next_index2: usize,
        // name: String,
    ) -> Self {
        Self {
            fop_id,
            next_index1,
            next_index2,
        }
    }
}

#[derive(Debug)]
pub struct Fops<T: Parses> {
    pub args: Vec<u8>,
    pub fops: Vec<Fop<T>>,
}

impl<T: Parses> Fops<T> {
    pub fn new(mut ir: Vec<Op<T>>) -> Self {
        if ir.len() >= u16::MAX as usize {
            panic!("Too Many Ops");
        }
        ir.push(Op::Matched);
        let mut args: Vec<u8> = Vec::new();
        let mut ir_fops: Vec<IRFop> = Vec::new();
        let mut fops: Vec<Fop<T>> = Vec::new();
        for (i, op) in ir.into_iter().enumerate() {
            let index = i;
            let args_index = args.len() as u16;
            match op {
                Op::Matched => {
                    ir_fops.push(IRFop::new(
                        MATCHED, 0, 0, // "Matched".to_string()
                    ));
                    fops.push(Fop::new(matched, MATCHED, args_index));
                }
                Op::Match(value) => {
                    args.extend(value.to_bytes());
                    ir_fops.push(IRFop::new(
                        MATCH,
                        index + 1,
                        0, // format!("Match({:?})", bytes),
                    ));
                    fops.push(Fop::new(try_match, MATCH, args_index));
                }
                Op::MatchAny => {
                    ir_fops.push(IRFop::new(
                        MATCH_ANY,
                        index + 1,
                        0, // "MatchAny".to_string()
                    ));
                    fops.push(Fop::new(match_any, MATCH_ANY, args_index));
                }
                Op::Jump(j) => {
                    let target = match j {
                        Jmp::Up(add) => index + add,
                        Jmp::Back(sub) => index - sub,
                    };
                    ir_fops.push(IRFop::new(
                        JUMP, target, 0, // format!("Jump({})", target)
                    ));
                    fops.push(Fop::new(jump, JUMP, args_index));
                }
                Op::Branch(j1, j2) => {
                    let t1 = match j1 {
                        Jmp::Up(add) => index + add,
                        Jmp::Back(sub) => index - sub,
                    };
                    let t2 = match j2 {
                        Jmp::Up(add) => index + add,
                        Jmp::Back(sub) => index - sub,
                    };
                    ir_fops.push(IRFop::new(
                        BRANCH, t1, t2, // format!("Branch({}, {})", t1, t2),
                    ));
                    fops.push(Fop::new(branch_off, BRANCH, args_index));
                }
                Op::Scope => {
                    ir_fops.push(IRFop::new(
                        SCOPE,
                        index + 1,
                        0, // "Scope".to_string()
                    ));
                    fops.push(Fop::new(scope, SCOPE, args_index));
                }
                Op::CommitScope => {
                    ir_fops.push(IRFop::new(
                        COMMIT_SCOPE,
                        index + 1,
                        0, // "CommitScope".to_string(),
                    ));
                    fops.push(Fop::new(commit_scope, COMMIT_SCOPE, args_index));
                }
                Op::KillScope => {
                    ir_fops.push(IRFop::new(
                        KILL_SCOPE, 0, 0, // "KillScope".to_string(),
                    ));
                    fops.push(Fop::new(kill_scope, KILL_SCOPE, args_index));
                }
                Op::Save => {
                    ir_fops.push(IRFop::new(
                        SAVE,
                        index + 1,
                        0, // "Save".to_string()
                    ));
                    fops.push(Fop::new(save, SAVE, args_index));
                }
                Op::Unsave => {
                    ir_fops.push(IRFop::new(
                        UNSAVE,
                        index + 1,
                        0, // "Unsave".to_string()
                    ));
                    fops.push(Fop::new(unsave, UNSAVE, args_index));
                }
                Op::StartTok => {
                    ir_fops.push(IRFop::new(
                        START_TOK,
                        index + 1,
                        0, // "StartTok".to_string()
                    ));
                    fops.push(Fop::new(start_tok, START_TOK, args_index));
                }
                Op::EndTok => {
                    ir_fops.push(IRFop::new(
                        END_TOK,
                        index + 1,
                        0, // "EndTok".to_string()
                    ));
                    fops.push(Fop::new(end_tok, END_TOK, args_index));
                }
                Op::StartLoop => {
                    ir_fops.push(IRFop::new(
                        START_LOOP,
                        index + 1,
                        0, // "StartLoop".to_string(),
                    ));
                    fops.push(Fop::new(start_loop, START_LOOP, args_index));
                }
                Op::EndLoop(jump_back, min, max) => {
                    args.extend(min.to_be_bytes());
                    args.extend(max.to_be_bytes());
                    ir_fops.push(IRFop::new(
                        END_LOOP,
                        index - jump_back,
                        index + 1, // format!("EndLoop({}, {})", min, max),
                    ));
                    fops.push(Fop::new(end_loop, END_LOOP, args_index));
                }
            }
        }
        for (i, irfop) in ir_fops.iter().enumerate() {
            match irfop.fop_id {
                MATCHED => {}
                BRANCH | END_LOOP => {
                    let [fop, next1, next2] = fops
                        .get_disjoint_mut([i, irfop.next_index1, irfop.next_index2])
                        .unwrap();
                    fop.next1 = next1;
                    fop.next2 = next2;
                }
                _ => {
                    let [fop, next1] = fops.get_disjoint_mut([i, irfop.next_index1]).unwrap();
                    fop.next1 = next1;
                }
            }
        }
        Self { args, fops }
    }

    pub fn get_first_fop(&self) -> *const Fop<T> {
        self.fops.first().unwrap()
    }

    pub fn call(&self, vm_ptr: *mut Parser<T>, snip: &Snip<T>, tid: u16) {
        unsafe {
            let vm = &mut *vm_ptr;
            let fop = &*vm.threads[tid].ip;
            fop.call(vm, snip, tid, Args::new(&self.args, fop.args_id));
        }
    }
}
