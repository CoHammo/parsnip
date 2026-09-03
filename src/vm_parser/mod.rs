pub mod compilers;
mod events;
mod iter;
mod scopes;
mod stack;
// mod fops;
mod ops;
mod tests;
mod threads;

pub use compilers::*;
use events::*;
// use fops::*;
use iter::*;
use ops::*;
use scopes::*;
use stack::*;
use threads::*;

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    Running,
    Matched,
    Failed,
}

pub struct Parser {
    stat: Stat,
    debug: bool,
    seen: Vec<ThreadState>,
    // seen: BTreeSet<ThreadState>,
    ops: Ops,
    scopes: Scopes,
    stack: Stack,
    threads: Threads,
    events: EventsBuilder,
    best_match: Option<u32>,
}

impl Parser {
    pub fn new<T: Parses>(ops: Vec<Op<T>>) -> Self {
        Self {
            stat: Stat::Running,
            debug: false,
            seen: Vec::new(),
            ops: Ops::new(ops),
            scopes: Scopes::new(),
            threads: Threads::new(),
            stack: Stack::new(),
            events: EventsBuilder::new(),
            best_match: None,
        }
    }

    pub fn debug(&mut self) {
        self.debug = true
        // self.threads.debug = self.debug;
    }

    fn was_seen(&mut self, id: u16, ip: u16) -> bool {
        let state = self.threads[id].get_state(ip);
        if self.seen.contains(&state) {
            true
        } else {
            self.seen.push(state);
            false
        }
    }

    fn kill_thread(&mut self, id: u16, unref_events: bool) {
        let thread = &mut self.threads[id];
        self.stack.unref(thread.stack);
        if unref_events {
            self.events.unref(thread.event);
        }
        self.threads.kill(id);
        // println!("Killed Thread {}", id);
    }

    pub fn parse<T: Parses, I: SnipIter<T>>(&mut self, source: impl AsSnips<T, I>) -> Events {
        let mut snips = source.snips(..);
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

    pub fn take_snip<T: Parses, I: SnipIter<T>>(&mut self, snip: &Snip<T>) {
        while let Some((id, mut ip)) = self.threads.next_thread()
            && self.stat == Stat::Running
        {
            if self.scopes.is_dead(self.threads[id].scope.val()) {
                self.kill_thread(id, true);
                continue;
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
                if self.was_seen(id, ip) {
                    self.kill_thread(id, true);
                    break;
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
                            let slice = self.ops.get_match_slice(ip);
                            if value.matches(slice) {
                                ip += (slice.len() + 1) as u16;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                let event = thread.event;
                                thread.rewind(&mut self.stack);
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
                        let target = self.ops.get_jump_target(ip);
                        ip = target;
                    }
                    BRANCH => {
                        let (target1, target2) = self.ops.get_branch_targets(ip);
                        ip = target1;
                        let fork = self.threads.fork_thread(id);
                        fork.ip = target2;
                        self.stack.upref(fork.stack);
                        self.events.upref(fork.event);
                    }
                    SCOPE => {
                        self.threads[id].scope.add_scope(self.scopes.next_scope());
                        ip += 1;
                    }
                    COMMIT_SCOPE => {
                        if let Some(scope_id) = self.threads[id].scope.pop_scope() {
                            self.scopes.kill_scope(scope_id);
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    KILL_SCOPE => {
                        if let Some(scope_id) = self.threads[id].scope.last_scope() {
                            self.scopes.kill_scope(scope_id);
                        } else {
                            println!("Tried to kill a scope that doesn't exist");
                            self.stat = Stat::Failed;
                        }
                        break;
                    }
                    SAVE => {
                        ip += 1;
                        let thread = &mut self.threads[id];
                        thread.stack = self
                            .stack
                            .push_stack(Var::save(ip, thread.event, thread.scope), thread.stack);
                        thread.saves += 1;
                    }
                    UNSAVE => {
                        let thread = &mut self.threads[id];
                        if let Some((prev, Var::Save { .. })) = self.stack.pop_stack(thread.stack) {
                            thread.stack = prev;
                            thread.saves -= 1;
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
                            let (start, min, max) = self.ops.get_loop_bounds(ip);
                            if loo.count == max {
                                thread.stack = self.stack.pop_stack(thread.stack).unwrap().0;
                                ip += 11;
                            } else {
                                if loo.count >= min {
                                    let fork = self.threads.fork_thread(id);
                                    fork.ip = ip + 11;
                                    fork.stack = self.stack.before(fork.stack);
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

        self.seen.clear();
        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
        }
    }
}
