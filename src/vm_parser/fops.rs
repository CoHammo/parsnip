use std::ops::Index;

use super::{Jmp, Op, Parser, Parses, Snip, Stat, Var};

pub struct Args {
    args: Vec<u8>,
}

impl Args {
    pub fn new(args: Vec<u8>) -> Self {
        Self { args }
    }

    pub fn get_match_slice(&self, index: u16) -> &[u8] {
        unsafe {
            let len = self[index] as u16;
            &self
                .args
                .get_unchecked((index + 1) as usize..((index + 1 + len) as usize))
        }
    }

    pub fn get_jump_target(&self, index: u16) -> u16 {
        let upper = self[index];
        let lower = self[index + 1];
        (upper as u16) << 8 | lower as u16
    }

    pub fn get_branch_targets(&self, index: u16) -> (u16, u16) {
        let upper1 = self[index];
        let lower1 = self[index + 1];
        let upper2 = self[index + 2];
        let lower2 = self[index + 3];
        (
            (upper1 as u16) << 8 | lower1 as u16,
            (upper2 as u16) << 8 | lower2 as u16,
        )
    }

    pub fn get_loop_bounds(&self, index: u16) -> (u32, u32) {
        let up1 = self[index];
        let upmid1 = self[index + 1];
        let lowmid1 = self[index + 2];
        let low1 = self[index + 3];
        let min = (up1 as u32) << 24 | (upmid1 as u32) << 16 | (lowmid1 as u32) << 8 | low1 as u32;

        let up2 = self[index + 4];
        let upmid2 = self[index + 5];
        let lowmid2 = self[index + 6];
        let low2 = self[index + 8];
        let max = (up2 as u32) << 24 | (upmid2 as u32) << 16 | (lowmid2 as u32) << 8 | low2 as u32;
        (min, max)
    }
}

impl Index<u16> for Args {
    type Output = u8;
    fn index(&self, index: u16) -> &u8 {
        unsafe { self.args.get_unchecked(index as usize) }
    }
}

pub fn matched<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    if let Some(best) = vm.best_match
        && best != 0
    {
        vm.events.unref(best);
    }
    vm.best_match = Some(thread.event);
    vm.kill_thread(tid, false);
    false
}

pub fn try_match<T: Parses>(
    vm: &mut Parser,
    snip: &Snip<T>,
    tid: u16,
    args: &Args,
    args_id: u16,
) -> bool {
    if let Some(value) = &snip.value {
        let slice = args.get_match_slice(args_id);
        if value.matches(slice) {
            vm.threads[tid].ip += 1;
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
    false
}

pub fn match_any<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    vm.threads[tid].ip += 1;
    false
}

pub fn jump<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, args: &Args, args_id: u16) -> bool {
    let target = args.get_jump_target(args_id);
    vm.threads[tid].ip = target;
    true
}

pub fn branch_off<T: Parses>(
    vm: &mut Parser,
    _: &Snip<T>,
    tid: u16,
    args: &Args,
    args_id: u16,
) -> bool {
    let (t1, t2) = args.get_branch_targets(args_id);
    vm.threads[tid].ip = t1;
    let fork = vm.threads.fork_thread(tid);
    fork.ip = t2;
    vm.stack.upref(fork.stack);
    vm.events.upref(fork.event);
    true
}

pub fn scope<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    thread.scope.add_scope(vm.scopes.next_scope());
    thread.ip += 1;
    true
}

pub fn commit_scope<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    if let Some(scope_id) = thread.scope.pop_scope() {
        vm.scopes.kill_scope(scope_id);
        thread.ip += 1;
    } else {
        println!("Tried to commit a scope that doesn't exist");
        vm.stat = Stat::Failed;
        return false;
    }
    true
}

pub fn kill_scope<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    if let Some(scope_id) = thread.scope.last_scope() {
        vm.scopes.kill_scope(scope_id);
    } else {
        println!("Tried to kill a scope that doesn't exist");
        vm.stat = Stat::Failed;
        return false;
    }
    true
}

pub fn save<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    thread.ip += 1;
    thread.stack = vm.stack.push_stack(
        Var::save(thread.ip, thread.event, thread.scope),
        thread.stack,
    );
    thread.saves += 1;
    true
}

pub fn unsave<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    if let Some((prev, Var::Save { .. })) = vm.stack.pop_stack(thread.stack) {
        thread.stack = prev;
        thread.saves -= 1;
        thread.ip += 1;
    } else {
        println!("Tried to unsave without a save");
        vm.stat = Stat::Failed;
        return false;
    }
    true
}

pub fn start_tok<T: Parses>(vm: &mut Parser, snip: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    thread.event = vm.events.push_event(true, snip.index, thread.event);
    thread.ip += 1;
    true
}

pub fn end_tok<T: Parses>(vm: &mut Parser, snip: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    thread.event = vm.events.push_event(false, snip.index, thread.event);
    thread.ip += 1;
    true
}

pub fn start_loop<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    thread.ip += 1;
    thread.stack = vm.stack.push_stack(Var::loo(thread.ip), thread.stack);
    true
}

pub fn end_loop<T: Parses>(vm: &mut Parser, _: &Snip<T>, tid: u16, _: &Args, _: u16) -> bool {
    let thread = &mut vm.threads[tid];
    if let Some((new_stack_id, Var::Loop(loo))) = vm.stack.edit(thread.stack) {
        thread.stack = new_stack_id;
        loo.count += 1;
        let (min, max) = vm.ops.get_loop_bounds(thread.ip);
        if loo.count == max {
            thread.stack = vm.stack.pop_stack(thread.stack).unwrap().0;
            thread.ip += 1;
        } else {
            let fork_ip = thread.ip + 1;
            thread.ip = loo.start;
            if loo.count >= min {
                let fork = vm.threads.fork_thread(tid);
                fork.ip = fork_ip;
                fork.stack = vm.stack.before(fork.stack);
                vm.stack.upref(fork.stack);
                vm.events.upref(fork.event);
            }
        }
    } else {
        println!("Tried to close a loop with no start");
        vm.stat = Stat::Failed;
        return false;
    }
    true
}

pub struct FOp<T: Parses> {
    pub fun: fn(vm: &mut Parser, snip: &Snip<T>, tid: u16, args: &Args, args_id: u16) -> bool,
    pub args_id: u16,
}

impl<T: Parses> FOp<T> {
    pub fn new(
        fun: fn(vm: &mut Parser, snip: &Snip<T>, tid: u16, args: &Args, args_id: u16) -> bool,
        args_id: u16,
    ) -> Self {
        Self { fun, args_id }
    }
}

pub struct FOps<T: Parses> {
    args: Args,
    fops: Vec<FOp<T>>,
}

impl<T: Parses> FOps<T> {
    pub fn new(mut ir: Vec<Op<T>>) -> Self {
        if ir.len() >= u16::MAX as usize {
            panic!("Too Many Ops");
        }
        ir.push(Op::Matched);
        let mut args: Vec<u8> = Vec::new();
        let mut fops: Vec<FOp<T>> = Vec::new();
        for (i, op) in ir.into_iter().enumerate() {
            let index = i as u16;
            let args_index = args.len() as u16;
            match op {
                Op::Matched => {
                    fops.push(FOp::new(matched::<T>, args_index));
                }
                Op::Match(value) => {
                    let bytes = value.to_bytes();
                    args.push(bytes.len() as u8);
                    args.extend(bytes);
                    fops.push(FOp::new(try_match::<T>, args_index));
                }
                Op::MatchAny => {
                    fops.push(FOp::new(match_any::<T>, args_index));
                }
                Op::Jump(j) => {
                    let target = match j {
                        Jmp::Up(add) => index + add,
                        Jmp::Back(sub) => index - sub,
                    };
                    let upper = (target >> 8) as u8;
                    let lower = target as u8;
                    args.extend([upper, lower]);
                    fops.push(FOp::new(jump::<T>, args_index));
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
                    let upper1 = (t1 >> 8) as u8;
                    let lower1 = t1 as u8;
                    let upper2 = (t2 >> 8) as u8;
                    let lower2 = t2 as u8;
                    args.extend([upper1, lower1, upper2, lower2]);
                    fops.push(FOp::new(branch_off::<T>, args_index));
                }
                Op::Scope => {
                    fops.push(FOp::new(scope::<T>, args_index));
                }
                Op::CommitScope => {
                    fops.push(FOp::new(commit_scope::<T>, args_index));
                }
                Op::KillScope => {
                    fops.push(FOp::new(kill_scope::<T>, args_index));
                }
                Op::Save => {
                    fops.push(FOp::new(save::<T>, args_index));
                }
                Op::Unsave => {
                    fops.push(FOp::new(unsave::<T>, args_index));
                }
                Op::StartTok => {
                    fops.push(FOp::new(start_tok::<T>, args_index));
                }
                Op::EndTok => {
                    fops.push(FOp::new(end_tok::<T>, args_index));
                }
                Op::StartLoop => {
                    fops.push(FOp::new(start_loop::<T>, args_index));
                }
                Op::EndLoop(min, max) => {
                    args.extend(min.to_be_bytes());
                    args.extend(max.to_be_bytes());
                    fops.push(FOp::new(end_loop::<T>, args_index));
                }
            }
        }
        // args.push(0);
        Self {
            args: Args { args },
            fops,
        }
    }

    pub fn run(&self, vm: &mut Parser, snip: &Snip<T>, tid: u16) {
        let mut fop = &self.fops[vm.threads[tid].ip as usize];
        while (fop.fun)(vm, snip, tid, &self.args, fop.args_id) {
            fop = &self.fops[vm.threads[tid].ip as usize];
        }
    }
}
