use std::collections::HashSet;

use super::super::*;

parser!(Alt alt {
    parsers: Vec<Parser> => inners: Box<[Parser]>,
    => running_inners: HashSet<usize>,
} {
    running_inners = (0..parsers.len()).collect();
    inners = parsers.into_boxed_slice();
});

impl Clone for Alt {
    fn clone(&self) -> Self {
        Alt::new(self.inners.to_vec())
    }
}

impl CharParser for Alt {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        for index in self.running_inners.clone() {
            match self.inners.get_mut(index) {
                Some(parser) => match parser.take_char(ch) {
                    Stat::Running => {}
                    Stat::HasMatch(end_byte) => {
                        self.stat = Stat::HasMatch(end_byte);
                    }
                    Stat::Matched(end_byte) => {
                        let toks = parser.take_tokens();
                        self.add_tokens(toks);
                        self.stat = Stat::Matched(end_byte);
                        break;
                    }
                    Stat::Failed => {
                        self.running_inners.take(&index);
                        if self.running_inners.is_empty() {
                            self.stat = Stat::Failed;
                            break;
                        }
                    }
                },
                None => self.stat = Stat::Failed,
            }
        }
        self.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        for runi in self.running_inners.clone().iter() {
            match self.inners.get_mut(*runi) {
                Some(parser) => match parser.finish(ch) {
                    Stat::Matched(end_byte) => {
                        let toks = parser.take_tokens();
                        self.add_tokens(toks);
                        self.stat = Stat::Matched(end_byte);
                        break;
                    }
                    _ => self.stat = Stat::Failed,
                },
                None => self.stat = Stat::Failed,
            }
        }
        self.stat
    }

    fn reset(&mut self) {
        self.reset_base();
        self.running_inners = (0..self.inners.len()).collect();
        for parser in self.inners.iter_mut() {
            parser.reset();
        }
    }

    fn string(&self) -> String {
        format!(
            "Alt([{}])",
            self.inners
                .iter()
                .map(|p| p.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
