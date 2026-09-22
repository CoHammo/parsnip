pub mod compilers;
mod events;
mod ops;
mod peeks;
mod scopes;
mod stack;
mod tests;
mod threads;

use super::types::iter::*;
pub use compilers::*;
use events::*;
use ops::*;
use peeks::*;
use scopes::*;
use stack::*;
use threads::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    Running,
    Matched,
    Failed,
}

#[derive(Debug)]
pub struct Parser {
    stat: Stat,
    debug: bool,
    // seen: Vec<ThreadState>,
    ops: Ops,
    scopes: ScopeStack,
    peeks: PeekStack,
    stack: Stack,
    threads: Threads,
    events: EventsBuilder,
    best_match: Option<u32>,
}

impl Parser {
    pub fn new<T: Parses>(ops: Vec<Op<T>>) -> Self {
        let cops = Ops::new(ops);
        let me = Self {
            stat: Stat::Running,
            debug: false,
            // seen: Vec::new(),
            ops: cops,
            scopes: ScopeStack::new(),
            peeks: PeekStack::new(),
            threads: Threads::new(),
            stack: Stack::new(),
            events: EventsBuilder::new(),
            best_match: None,
        };
        me
    }

    pub fn debug(&mut self) {
        self.debug = true
        // self.threads.debug = self.debug;
    }

    fn fork(&mut self, id: u16, ip: u16) -> (u16, &mut Thread) {
        let (fork_id, fork) = self.threads.fork_thread(id);
        fork.ip = ip;
        self.peeks.upref(fork.peek);
        self.scopes.upref(fork.scope);
        self.stack.upref(fork.stack);
        self.events.upref(fork.event);
        (fork_id, fork)
    }

    fn kill_thread(&mut self, id: u16, free_events: bool) {
        let thread = &mut self.threads[id];
        self.scopes.unref(thread.scope);
        self.peeks.unref(thread.peek);
        self.stack.unref(thread.stack);
        if free_events {
            self.events.unref(thread.event);
        }
        self.threads.kill_thread(id);
        // println!("Killed Thread {}", id);
    }

    pub fn parse<T: Parses, I: SnipsIter<T>>(&mut self, source: impl AsSnips<T, I>) -> Events {
        let mut snips = source.snips();
        while let Some(snip) = snips.next() {
            self.take_snip::<T, I>(&snip);
        }

        if let Some(best) = self.best_match {
            self.stat = Stat::Matched;
            if best == 0 {
                Events::empty()
            } else {
                self.events.build_from(best)
            }
        } else {
            self.stat = Stat::Failed;
            Events::empty()
        }
    }

    pub fn take_snip<T: Parses, I: SnipsIter<T>>(&mut self, snip: &Snip<T, I>) {
        while let Some((id, mut ip)) = self.threads.next_thread()
            && self.stat == Stat::Running
        {
            let th = &mut self.threads[id];
            let peek = &mut self.peeks[th.peek];
            if (th.scope != 0 && !self.scopes[th.scope].alive) || peek.stat == PeekStat::Kill {
                self.kill_thread(id, true);
                continue;
            } else if peek.stat == PeekStat::Remove {
                if th.peek % 2 == 0 {
                    self.peeks.pop_peek(th.peek);
                } else {
                    self.kill_thread(id, true);
                    continue;
                }
            }
            loop {
                if self.debug {
                    println!(
                        "Thread {}: snipdex={}, snip={:?}, op={}",
                        id,
                        snip.index,
                        snip.value,
                        self.ops.get_info_at(ip).1
                    );
                }
                match self.ops[ip] {
                    MATCHED => {
                        let thread = &mut self.threads[id];
                        if let Some(best) = self.best_match
                            && best != 0
                        {
                            self.events.unref(best);
                        }
                        self.best_match = Some(thread.event);
                        self.kill_thread(id, false);
                        break;
                    }
                    MATCH => {
                        if let Some(value) = &snip.value {
                            let thread = &mut self.threads[id];
                            let slice = self.ops.get_match_args(ip);
                            if value.is_match(slice) {
                                ip += (slice.len() + 1) as u16;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                let event = thread.event;
                                thread.rewind(&mut self.stack, &mut self.scopes);
                                self.events.upref(thread.event);
                                self.events.unref(event);
                            } else {
                                self.kill_thread(id, true);
                            }
                        } else {
                            self.kill_thread(id, true);
                        }
                        break;
                    }
                    MATCH_ANY => {
                        ip += 1;
                        break;
                    }
                    JUMP => {
                        let target = self.ops.get_jump_args(ip);
                        ip = target;
                    }
                    BRANCH => {
                        let (target1, target2) = self.ops.get_branch_args(ip);
                        ip = target1;
                        self.fork(id, target2);
                    }
                    SCOPE => {
                        let thread = &mut self.threads[id];
                        thread.scope = self.scopes.add_scope(thread.scope);
                        ip += 1;
                    }
                    COMMIT_SCOPE => {
                        let thread = &mut self.threads[id];
                        if let Some(prev_scope) = self.scopes.pop_scope(thread.scope) {
                            self.scopes.kill_scope(thread.scope);
                            thread.scope = prev_scope;
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    KILL_SCOPE => {
                        self.scopes.kill_scope(self.threads[id].scope);
                        self.kill_thread(id, true);
                        break;
                    }
                    PEEK => {
                        let thread = &mut self.threads[id];
                        let (positive, target) = self.ops.get_peek_args(ip);
                        thread.peek = self.peeks.add_peek(thread.peek, positive);
                        let (_, fork) = self.fork(id, ip + 1);
                        fork.peek += 1;
                        ip = target;
                    }
                    COMMIT_PEEK => {
                        self.peeks.commit_peek(self.threads[id].peek);
                        self.kill_thread(id, true);
                    }
                    SAVE => {
                        ip += 1;
                        let thread = &mut self.threads[id];
                        thread.stack = self.stack.push_stack(
                            Var::save(ip, thread.event, thread.scope, thread.peek),
                            thread.stack,
                        );
                        thread.saves += 1;
                    }
                    UNSAVE => {
                        let thread = &mut self.threads[id];
                        if let Some((prev, Var::Save { .. })) = self.stack.pop_stack(thread.stack) {
                            thread.stack = prev;
                            if thread.saves != 0 {
                                thread.saves -= 1;
                            }
                            ip += 1;
                        } else {
                            println!("Tried to unsave without a save");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    START_TOK => {
                        let thread = &mut self.threads[id];
                        thread.event = self.events.push_event(true, snip.index, thread.event);
                        ip += 1;
                    }
                    END_TOK => {
                        let thread = &mut self.threads[id];
                        thread.event = self.events.push_event(false, snip.index, thread.event);
                        ip += 1;
                    }
                    START_LOOP => {
                        let thread = &mut self.threads[id];
                        thread.stack = self.stack.push_stack(Var::loo(), thread.stack);
                        ip += 1;
                    }
                    END_LOOP => {
                        let thread = &mut self.threads[id];
                        if let Some((new_stack_id, Var::Loop(loo))) = self.stack.edit(thread.stack)
                        {
                            thread.stack = new_stack_id;
                            loo.count += 1;
                            let (start, min, max) = self.ops.get_loop_args(ip);
                            if loo.count == max {
                                thread.stack = self.stack.pop_stack(thread.stack).unwrap().0;
                                ip += 11;
                            } else {
                                if loo.count >= min {
                                    let (_, fork) = self.threads.fork_thread(id);
                                    fork.ip = ip + 11;
                                    fork.stack = self.stack.before(fork.stack);
                                    self.scopes.upref(fork.scope);
                                    self.stack.upref(fork.stack);
                                    self.events.upref(fork.event);
                                }
                                ip = start;
                            }
                        } else {
                            println!("Tried to close a loop with no start");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    op => {
                        panic!("Bad Op!! {}", op);
                    }
                }
                if self.debug {
                    println!("    {}", self.threads[id].dbg());
                }
            } // Command Loop
            self.threads[id].ip = ip;
        } // Threads Loop

        // self.seen.clear();
        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
        }
    }
}
