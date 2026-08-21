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
use vec_linked_list::*;

pub struct Parser<T: Matches> {
    stat: Stat,
    debug: bool,
    comms: Vec<Comm<T>>,
    threads: Threads,
    next_scope: usize,
    // seen: HashSet<usize>,
    events: EventsBuilder,
    best_match: Option<Link<Event>>,
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
            Some(event_link) => {
                self.stat = Stat::Matched;
                self.events.build_from(&event_link)
            }
            None => {
                self.stat = Stat::Failed;
                Events::empty()
            }
        }
    }

    pub fn take_snip<I: SnipIter<T>>(&mut self, snip: &Snip<T, I>) {
        while let Some(link) = self.threads.next()
            && self.stat == Stat::Running
        {
            let thread = link.val();
            loop {
                if self.debug {
                    println!(
                        "Thread {:?}: snipdex={}, snip={:?}, ip={}, comm={:?}",
                        link, snip.index, snip.value, thread.ip, self.comms[thread.ip]
                    );
                }
                match unsafe { self.comms.get_unchecked(thread.ip) } {
                    Comm::Matched => {
                        self.best_match = thread.event;
                        self.threads.free(link);
                        break;
                    }
                    Comm::Match(thing) => {
                        if let Some(value) = &snip.value {
                            if thing.matches(value) {
                                thread.ip += 1;
                            } else if thread.saves > 0 {
                                if self.debug {
                                    println!("    Rewinding...");
                                }
                                thread.rewind();
                            } else {
                                self.threads.free(link);
                            }
                        } else {
                            self.threads.free(link);
                        }
                        break;
                    }
                    Comm::MatchAny => {
                        thread.ip += 1;
                        break;
                    }
                    &Comm::Jump(up, num) => {
                        thread.ip = match up {
                            true => thread.ip + num,
                            false => thread.ip - num,
                        }
                    }
                    &Comm::Branch(up1, num1, up2, num2) => {
                        let fork = self.threads.fork(&link);
                        fork.ip = match up2 {
                            true => thread.ip + num2,
                            false => thread.ip - num2,
                        };
                        thread.ip = match up1 {
                            true => thread.ip + num1,
                            false => thread.ip - num1,
                        };
                    }
                    Comm::Scope => {
                        let scope = self.new_scope();
                        thread.state.push(State::Scope(scope));
                        thread.ip += 1;
                    }
                    Comm::CommitScope => {
                        if let Some(State::Scope(scope)) = thread.state.pop() {
                            self.threads.kill_scope(scope);
                            thread.ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::KillScope => {
                        if let Some(State::Scope(scope)) = thread.state.last() {
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
                        thread.ip += 1;
                        thread.state.push(State::Save {
                            ip: thread.ip,
                            event: thread.event,
                        });
                        thread.saves += 1;
                    }
                    Comm::Unsave => {
                        if let Some(State::Save { .. }) = thread.state.pop() {
                            thread.saves -= 1;
                            thread.ip += 1;
                        } else {
                            println!("Tried to unsave without a save");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    &Comm::Tok(start) => {
                        thread.event = Some(self.events.push(start, snip.index, thread.event));
                        thread.ip += 1;
                    }
                    Comm::StartLoop => {
                        thread.ip += 1;
                        thread.state.push(State::Loop(Loop::new(thread.ip)));
                    }
                    &Comm::EndLoop(min, max) => {
                        if let Some(State::Loop(loo)) = thread.state.last_mut() {
                            loo.count += 1;
                            if loo.count == max {
                                thread.state.pop();
                                thread.ip += 1;
                            } else {
                                if loo.count >= min {
                                    let fork = self.threads.fork(&link);
                                    fork.ip = thread.ip + 1;
                                    fork.state.pop();
                                }
                                thread.ip = loo.start;
                            }
                        } else {
                            println!("Tried to close a loop with no start");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                }
                if self.debug {
                    println!("    {}", thread.dbg());
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
