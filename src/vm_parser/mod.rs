pub mod compiler;
mod events;
mod iter;
mod linked_vec;
mod tests;
mod threads;
mod types;

pub use compiler::*;
use events::*;
use iter::*;
use linked_vec::*;
use threads::*;
use types::*;

pub struct Parser<T: Matches> {
    stat: Stat,
    debug: bool,
    comms: Vec<Comm<T>>,
    threads: Threads,
    next_scope: usize,
    // seen: HashSet<usize>,
    events: EventsBuilder,
    best_match: Option<usize>,
}

impl<T: Matches> Parser<T> {
    pub fn new(mut comms: Vec<Comm<T>>) -> Self {
        comms.push(Comm::Matched);
        Self {
            stat: Stat::Running,
            debug: false,
            comms,
            threads: Threads::new(),
            next_scope: 0,
            // seen: HashSet::new(),
            events: EventsBuilder::new(),
            best_match: None,
        }
    }

    pub fn debug(&mut self) {
        self.debug = true
        // self.threads.debug = self.debug;
    }

    pub fn new_scope(&mut self) -> usize {
        let s = self.next_scope;
        self.next_scope += 1;
        s
    }

    pub fn parse<I: SnipIter<T>>(&mut self, source: impl AsSnips<T, I>) -> Events {
        let mut snips = source.as_snips(..);
        while let Some(snip) = snips.next()
            && self.stat == Stat::Running
        {
            self.take_snip(&snip);
        }
        match self.best_match {
            Some(event_link) => {
                self.stat = Stat::Matched;
                self.events.build_from(event_link)
            }
            None => {
                self.stat = Stat::Failed;
                Events::empty()
            }
        }
    }

    pub fn take_snip<I: SnipIter<T>>(&mut self, snip: &Snip<T, I>) {
        while let Some((id, mut ip)) = self.threads.next()
            && self.stat == Stat::Running
        {
            loop {
                if self.debug {
                    println!(
                        "Thread {}: snipdex={}, snip={:?}, ip={}, comm={:?}",
                        id, snip.index, snip.value, ip, self.comms[ip]
                    );
                }
                match unsafe { self.comms.get_unchecked(ip) } {
                    Comm::Matched => {
                        self.best_match = Some(self.threads[id].event);
                        self.threads.kill(id);
                        break;
                    }
                    Comm::Match(thing) => {
                        let thread = &mut self.threads[id];
                        if let Some(value) = &snip.value {
                            if thing.matches(value) {
                                ip += 1;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                let event = thread.event;
                                thread.rewind();
                                self.events.upref(thread.event);
                                self.events.unref(event);
                            } else {
                                self.events.unref(thread.event);
                                self.threads.kill(id);
                            }
                        } else {
                            self.events.unref(thread.event);
                            self.threads.kill(id);
                        }
                        break;
                    }
                    Comm::MatchAny => {
                        ip += 1;
                        break;
                    }
                    &Comm::Jump(up, num) => {
                        ip = match up {
                            true => ip + num,
                            false => ip - num,
                        }
                    }
                    &Comm::Branch(up1, num1, up2, num2) => {
                        self.threads.fork(id, |fork| {
                            fork.ip = match up2 {
                                true => ip + num2,
                                false => ip - num2,
                            };
                            self.events.upref(fork.event);
                        });
                        ip = match up1 {
                            true => ip + num1,
                            false => ip - num1,
                        };
                    }
                    Comm::Scope => {
                        let scope = self.new_scope();
                        self.threads[id].state.push(State::Scope(scope));
                        ip += 1;
                    }
                    Comm::CommitScope => {
                        if let Some(State::Scope(scope)) = self.threads[id].state.pop() {
                            self.threads
                                .kill_scope(scope, |t| self.events.unref(t.event));
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::KillScope => {
                        if let Some(State::Scope(scope)) = self.threads[id].state.last() {
                            self.threads
                                .kill_scope(*scope, |t| self.events.unref(t.event));
                            break;
                        } else {
                            println!("Tried to kill a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::Save => {
                        ip += 1;
                        let thread = &mut self.threads[id];
                        thread.state.push(State::Save {
                            ip,
                            event: thread.event,
                        });
                        thread.saves += 1;
                    }
                    Comm::Unsave => {
                        let thread = &mut self.threads[id];
                        if let Some(State::Save { .. }) = thread.state.pop() {
                            thread.saves -= 1;
                            ip += 1;
                        } else {
                            println!("Tried to unsave without a save");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    &Comm::Tok(start) => {
                        let thread = &mut self.threads[id];
                        thread.event = self.events.add(start, snip.index, thread.event);
                        ip += 1;
                    }
                    Comm::StartLoop => {
                        ip += 1;
                        self.threads[id].state.push(State::Loop(Loop::new(ip)));
                    }
                    &Comm::EndLoop(min, max) => {
                        let thread = &mut self.threads[id];
                        if let Some(State::Loop(loo)) = thread.state.last_mut() {
                            loo.count += 1;
                            if loo.count == max {
                                thread.state.pop();
                                ip += 1;
                            } else {
                                if loo.count >= min {
                                    let fork_ip = ip + 1;
                                    ip = loo.start;
                                    self.threads.fork(id, |fork| {
                                        fork.ip = fork_ip;
                                        fork.state.pop();
                                        self.events.upref(fork.event);
                                    });
                                }
                            }
                        } else {
                            println!("Tried to close a loop with no start");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                }
                if self.debug {
                    println!("    {}", self.threads[id].dbg());
                }
            } // Command Loop
            self.threads[id].ip = ip;
        } // Threads Loop

        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
        }
    }
}
