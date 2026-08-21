pub mod compiler;
mod events;
mod iter;
mod tests;
mod threads;
mod types;
mod vec_linked_list;

pub use compiler::*;
use events::*;
use iter::*;
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

    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug;
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
            Some(event_id) => {
                self.stat = Stat::Matched;
                self.events.build_from(event_id)
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
                        let thread = self.threads.at(id);
                        thread.ip = ip;
                        self.best_match = thread.event;
                        self.threads.free(id);
                        break;
                    }
                    Comm::Match(thing) => {
                        let thread = self.threads.at(id);
                        if let Some(value) = &snip.value {
                            if thing.matches(value) {
                                thread.ip = ip + 1;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                thread.rewind();
                            } else {
                                self.threads.free(id);
                            }
                        } else {
                            self.threads.free(id);
                        }
                        break;
                    }
                    Comm::MatchAny => {
                        self.threads.at(id).ip = ip + 1;
                        break;
                    }
                    &Comm::Jump(up, num) => {
                        ip = match up {
                            true => ip + num,
                            false => ip - num,
                        }
                    }
                    &Comm::Branch(up1, num1, up2, num2) => {
                        self.threads.fork(id, |thread| {
                            thread.ip = match up2 {
                                true => ip + num2,
                                false => ip - num2,
                            };
                        });
                        ip = match up1 {
                            true => ip + num1,
                            false => ip - num1,
                        };
                    }
                    Comm::Scope => {
                        let scope = self.new_scope();
                        self.threads.at(id).state.push(State::Scope(scope));
                        ip += 1;
                    }
                    Comm::CommitScope => {
                        if let Some(State::Scope(scope)) = self.threads.at(id).state.pop() {
                            self.threads.kill_scope(scope);
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::KillScope => {
                        if let Some(State::Scope(scope)) = self.threads.at(id).state.last() {
                            let s = *scope;
                            self.threads.kill_scope(s);
                            break;
                        } else {
                            println!("Tried to kill a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::Save => {
                        let thread = self.threads.at(id);
                        ip += 1;
                        thread.state.push(State::Save {
                            ip,
                            last_event: thread.event,
                        });
                        thread.saves += 1;
                    }
                    Comm::Unsave => {
                        let thread = self.threads.at(id);
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
                        let thread = self.threads.at(id);
                        thread.event = Some(self.events.push(start, snip.index, thread.event));
                        ip += 1;
                    }
                    Comm::StartLoop => {
                        let thread = self.threads.at(id);
                        thread.state.push(State::Loop(Loop::new(ip + 1)));
                        ip += 1;
                    }
                    &Comm::EndLoop(min, max) => {
                        let thread = self.threads.at(id);
                        if let Some(State::Loop(loo)) = thread.state.last_mut() {
                            loo.count += 1;
                            if loo.count == max {
                                thread.state.pop();
                                ip += 1;
                            } else {
                                let fork_ip = ip + 1;
                                ip = loo.start;
                                if loo.count >= min {
                                    self.threads.fork(id, |fork| {
                                        fork.ip = fork_ip;
                                        fork.state.pop();
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
                    println!("    {}", self.threads.at(id).dbg());
                }
            } // Command Loop
        } // Threads Loop

        if !self.threads.restart() {
            self.stat = match self.best_match {
                Some(_) => Stat::Matched,
                None => Stat::Failed,
            }
        }
    }
}
