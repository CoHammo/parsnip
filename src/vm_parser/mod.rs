pub mod compiler;
mod events;
mod iter;
mod scopes;
mod state;
mod tests;
mod threads;

pub use compiler::*;
use events::*;
use iter::*;
use scopes::*;
use state::*;
use threads::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stat {
    Running,
    Matched,
    Failed,
}

#[derive(Debug, Clone)]
pub enum Comm<T: Matches> {
    Matched,
    Match(T),
    MatchAny,
    Jump(bool, usize),
    Branch(bool, usize, bool, usize),
    Scope,
    CommitScope,
    KillScope,
    Tok(bool),
    Save,
    Unsave,
    StartLoop,
    EndLoop(usize, usize),
}

pub struct Parser<T: Matches> {
    stat: Stat,
    debug: bool,
    comms: Vec<Comm<T>>,
    threads: Threads,
    state: StateStack,
    scopes: Scopes,
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
            state: StateStack::new(),
            scopes: Scopes::new(),
            // seen: HashSet::new(),
            events: EventsBuilder::new(),
            best_match: None,
        }
    }

    pub fn debug(&mut self) {
        self.debug = true
        // self.threads.debug = self.debug;
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
            let th = &mut self.threads[id];
            if self.scopes.is_dead(th.scope.val()) {
                self.state.unref(th.state);
                self.events.unref(th.event);
                self.threads.kill(id);
                continue;
            }
            loop {
                if self.debug {
                    println!(
                        "Thread {}: snipdex={}, snip={:?}, ip={}, comm={:?}",
                        id, snip.index, snip.value, ip, self.comms[ip]
                    );
                }
                match unsafe { self.comms.get_unchecked(ip) } {
                    Comm::Matched => {
                        let thread = &mut self.threads[id];
                        self.best_match = Some(thread.event);
                        self.state.unref(thread.state);
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
                                // let scope = thread.scope.val();
                                thread.rewind(&mut self.state);
                                // self.scopes.kill_scopes(thread.scope.diff(scope));
                                self.events.upref(thread.event);
                                self.events.unref(event);
                            } else {
                                self.state.unref(thread.state);
                                self.events.unref(thread.event);
                                self.threads.kill(id);
                            }
                        } else {
                            self.state.unref(thread.state);
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
                        let fork = self.threads.fork(id);
                        fork.ip = match up2 {
                            true => ip + num2,
                            false => ip - num2,
                        };
                        self.state.upref(fork.state);
                        self.events.upref(fork.event);
                        ip = match up1 {
                            true => ip + num1,
                            false => ip - num1,
                        };
                    }
                    Comm::Scope => {
                        self.threads[id].scope.add_scope(self.scopes.next());
                        ip += 1;
                    }
                    Comm::CommitScope => {
                        if let Some(sid) = self.threads[id].scope.pop() {
                            self.scopes.kill(sid);
                            ip += 1;
                        } else {
                            println!("Tried to commit a scope that doesn't exist");
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                    Comm::KillScope => {
                        if let Some(sid) = self.threads[id].scope.last() {
                            self.scopes.kill(sid);
                        } else {
                            println!("Tried to kill a scope that doesn't exist");
                            self.stat = Stat::Failed;
                        }
                        break;
                    }
                    Comm::Save => {
                        ip += 1;
                        let thread = &mut self.threads[id];
                        thread.state = self.state.push(
                            State::Save {
                                ip,
                                event: thread.event,
                                scope: thread.scope,
                            },
                            thread.state,
                        );
                        thread.saves += 1;
                    }
                    Comm::Unsave => {
                        let thread = &mut self.threads[id];
                        if let (prev, state) = self.state.pop(thread.state)
                            && let State::Save { .. } = state
                        {
                            thread.state = prev;
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
                        thread.event = self.events.push(start, snip.index, thread.event);
                        ip += 1;
                    }
                    Comm::StartLoop => {
                        ip += 1;
                        let thread = &mut self.threads[id];
                        thread.state = self.state.push(State::Loop(Loop::new(ip)), thread.state);
                    }
                    &Comm::EndLoop(min, max) => {
                        let thread = &mut self.threads[id];
                        if let Some(State::Loop(_)) = self.state.check(thread.state) {
                            let mut start: usize = 0;
                            let mut count: usize = 0;
                            let (state_id, st) = self.state.edit(thread.state);
                            thread.state = state_id;
                            if let State::Loop(loo) = st {
                                start = loo.start;
                                loo.count += 1;
                                count = loo.count;
                            }
                            if count == max {
                                let (stid, _) = self.state.pop(thread.state);
                                thread.state = stid;
                                ip += 1;
                            } else {
                                let fork_ip = ip + 1;
                                ip = start;
                                if count >= min {
                                    let fork = self.threads.fork(id);
                                    fork.ip = fork_ip;
                                    fork.state = self.state.before(fork.state);
                                    self.state.upref(fork.state);
                                    self.events.upref(fork.event)
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
