mod macros;
mod tests;
mod types;

use std::sync::Arc;
use types::*;
use unicode_segmentation::UnicodeSegmentation;
use wasm_bindgen::prelude::*;

make_parsers!(Str, Tok, Run, Rep, Till);

trait CharParser: Clone {
    fn take_char(&mut self, ch: &Char) -> Stat;
    fn finish(&mut self, ch: &Char) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}

parser!(Str str {
    value: String,
    => chars: Vec<Box<str>>,
    => len: usize,
    char_index: usize = 0,
} (keys) {
    keys = if let Some(ch) = value.graphemes(true).next() {
        Some(Arc::new([ch.to_string().into_boxed_str()]))
    } else {
        None
    };
    chars = value
        .graphemes(true)
        .map(|c| c.to_string().into_boxed_str())
        .collect();
    len = chars.len();
});
impl Clone for Str {
    fn clone(&self) -> Self {
        Str::new(self.value.clone())
    }
}
impl CharParser for Str {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        if self.len == 0 {
            self.stat = Stat::Failed;
        } else {
            if *ch.value == *self.chars[self.char_index] {
                self.char_index += 1;
                if self.char_index == self.len {
                    self.stat = Stat::Matched(ch.next_byte_offset());
                }
            } else {
                self.stat = Stat::Failed;
            }
        }
        // log(&format!(
        //     "matching={:?} char={}, char_index={}, byte_offset={}, status={:?}",
        //     self.chars,
        //     ch.value.escape_default(),
        //     ch.char_index,
        //     ch.byte_offset,
        //     self.status
        // ));
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.len == 0 {
            self.stat = Stat::Matched(ch.next_byte_offset());
        } else {
            self.stat = Stat::Failed;
        };
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.char_index = 0;
    }

    fn string(&self) -> String {
        format!("Str({})", self.value)
    }
}

parser!(Tok tok {
    parser: Parser => inner: Box<Parser>,
    kind: Option<String>,
    save_value: Option<bool> => save: bool,
} (keys) {
    keys = parser.keys();
    inner = Box::new(parser);
    save = save_value.unwrap_or(true);
});
impl Clone for Tok {
    fn clone(&self) -> Self {
        Tok::new(*self.inner.clone(), self.kind.clone(), Some(self.save))
    }
}
impl Tok {
    fn tokenize(&mut self, from_string: &str, end_byte: usize) {
        self.tokens = Some(vec![Token::new(
            self.kind.clone(),
            match self.save {
                true => Some(&from_string[self.start_byte..end_byte]),
                false => None,
            },
            self.start_byte,
            end_byte,
            self.inner.take_tokens(),
        )]);
    }
}
impl CharParser for Tok {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        match self.inner.take_char(ch) {
            Stat::Matched(end_byte) => {
                self.tokenize(ch.full_string, end_byte);
                self.stat = Stat::Matched(end_byte);
            }
            stat => self.stat = stat,
        };
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.tokenize(ch.full_string, end_byte);
                self.stat = Stat::Matched(end_byte);
            }
            stat => self.stat = stat,
        };
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Tok({})", self.inner.string())
    }
}

parser!(Rep rep {
    parser: Parser => inner: Box<Parser>,
    min: Option<usize> => the_min: usize,
    max: Option<usize> => the_max: usize,
    count: usize = 0,
    end_byte: usize = 0,
} (keys) {
    keys = parser.keys();
    the_min = match min {
        Some(num) => match num {
            0 => 1,
            _ => num,
        },
        None => 1,
    };
    the_max = match max {
        Some(num) => {
            if num > 0 && num < the_min {
                the_min
            } else {
                num
            }
        }
        None => {
            if min.is_none() {
                0
            } else {
                the_min
            }
        }
    };
    inner = Box::new(parser);
});
impl Clone for Rep {
    fn clone(&self) -> Self {
        Rep::new(*self.inner.clone(), Some(self.the_min), Some(self.the_max))
    }
}
impl CharParser for Rep {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        match self.inner.take_char(&ch) {
            Stat::Matched(end_byte) => {
                self.count += 1;
                self.end_byte = end_byte;
                let toks = self.inner.take_tokens();
                self.add_tokens(toks);
                self.inner.reset();
                if self.count == self.the_max {
                    self.stat = Stat::Matched(end_byte);
                } else if self.count >= self.the_min {
                    self.stat = Stat::HasMatch(end_byte);
                }
            }
            Stat::Failed => {
                if self.count >= self.the_min {
                    self.stat = Stat::Matched(self.end_byte);
                } else {
                    self.stat = Stat::Failed;
                }
            }
            status => self.stat = status,
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.count += 1;
                self.end_byte = end_byte;
            }
            _ => {}
        }
        if self.count >= self.the_min {
            let toks = self.inner.take_tokens();
            self.add_tokens(toks);
            self.inner.reset();
            self.stat = Stat::Matched(self.end_byte);
        } else {
            self.stat = Stat::Failed;
        };
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
        self.count = 0;
        self.end_byte = 0;
    }

    fn string(&self) -> String {
        format!("Rep({})", self.inner.string())
    }
}

parser!(Run run {
    parsers: Vec<Parser> => inners: Vec<(Parser, Option<usize>)>,
    => len: usize,
    inner_index: usize = 0,
    inner_end_index: usize = 1,
} (keys) {
    keys = parsers[0].keys();
    inners = parsers.into_iter().map(|p| (p, None)).collect();
    len = inners.len();
});
impl Run {
    fn next_inner(&mut self) -> bool {
        let mut go = true;
        while self.inner_index < self.len - 1 && go {
            self.inner_index += 1;
            match self.inners[self.inner_index].0.stat() {
                Stat::Running => {
                    if self.inner_index == self.inner_end_index {
                        self.inner_end_index += 1;
                    }
                    go = false;
                }
                Stat::HasMatch(_) => go = false,
                Stat::Matched(_) => {
                    if self.inner_index == self.len - 1 {
                        return false;
                    }
                }
                Stat::Failed => {
                    self.stat = Stat::Failed;
                    go = false;
                }
            }
        }
        return !go;
    }

    fn set_end(&mut self, index: usize) {
        if index < self.len - 1 {
            self.inner_end_index = index + 2;
            let p = &mut self.inners[self.inner_end_index - 1];
            p.0.reset();
            p.1 = None;
        }
    }

    fn next_end(&mut self) {
        if self.inner_end_index < self.len {
            let p = &mut self.inners[self.inner_end_index];
            p.0.reset();
            p.1 = None;
            self.inner_end_index += 1;
        }
    }
}
impl Clone for Run {
    fn clone(&self) -> Self {
        Run::new(self.inners.clone().into_iter().map(|p| p.0).collect())
    }
}
impl CharParser for Run {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        for index in self.inner_index..self.inner_end_index {
            match self.inners.get_mut(index) {
                Some(parser) => match parser.0.take_char(ch) {
                    Stat::Running => {}
                    Stat::HasMatch(end_byte) => {
                        parser.1 = Some(end_byte);
                        self.set_end(index);
                        if index == self.len - 1 {
                            self.stat = Stat::HasMatch(end_byte);
                        }
                        break;
                    }
                    Stat::Matched(end_byte) => {
                        let mut should_break = false;
                        if let Some(last_end_byte) = parser.1 {
                            if last_end_byte != end_byte {
                                should_break = true;
                            }
                        }
                        if index == self.inner_index {
                            let toks = parser.0.take_tokens();
                            self.add_tokens(toks);
                            if !self.next_inner() {
                                self.stat = Stat::Matched(end_byte);
                            }
                        } else if index == self.inner_end_index - 1 {
                            self.next_end();
                        }
                        if should_break {
                            break;
                        }
                    }
                    Stat::Failed => {
                        if index == self.inner_index {
                            self.stat = Stat::Failed;
                        }
                    }
                },
                None => self.stat = Stat::Failed,
            }
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.inner_index == self.len - 1 {
            if let Some(p) = self.inners.get_mut(self.inner_index) {
                match p.0.finish(ch) {
                    Stat::Matched(end_byte) => {
                        let toks = p.0.take_tokens();
                        self.add_tokens(toks);
                        self.stat = Stat::Matched(end_byte)
                    }
                    _ => self.stat = Stat::Failed,
                }
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        for p in &mut self.inners[..=self.inner_index] {
            p.0.reset();
            p.1 = None;
        }
        self.inner_index = 0;
    }

    fn string(&self) -> String {
        format!(
            "Run([{}])",
            self.inners
                .iter()
                .map(|p| p.0.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

parser!(Till till {
    parser: Parser => inner: Box<Parser>,
    match_at_end: Option<bool> => match_eof: bool,
} (keys) {
    keys = parser.keys();
    match_eof = match_at_end.unwrap_or(false);
    inner = Box::new(parser);
});
impl Clone for Till {
    fn clone(&self) -> Self {
        Till::new(*self.inner.clone(), Some(self.match_eof))
    }
}
impl CharParser for Till {
    fn take_char(&mut self, ch: &Char) -> Stat {
        self.fresh_check(ch.byte_offset);
        match self.inner.take_char(ch) {
            Stat::Matched(end_byte) => {
                let toks = self.inner.take_tokens();
                self.add_tokens(toks);
                self.stat = Stat::Matched(end_byte)
            }
            Stat::Failed => {
                self.inner.reset();
            }
            stat => self.stat = stat,
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        match self.inner.finish(ch) {
            Stat::Matched(end_byte) => {
                self.stat = Stat::Matched(end_byte);
            }
            _ => {
                if self.match_eof {
                    self.stat = Stat::Matched(ch.next_byte_offset())
                } else {
                    self.stat = Stat::Failed;
                }
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.inner.reset();
    }

    fn string(&self) -> String {
        format!("Till({})", self.inner.string())
    }
}
