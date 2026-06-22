use super::super::*;
use icu_normalizer::ComposingNormalizer;

#[derive(Debug)]
pub struct Str {
    pub base: BaseParser,
    chars: Box<[char]>,
    len: usize,
    index: usize,
}
impl Str {
    pub fn new(value: &str) -> Self {
        let chars = ComposingNormalizer::new_nfc()
            .normalize(value)
            .chars()
            .collect::<Box<[char]>>();
        let len = chars.len();
        Self {
            base: BaseParser::new(),
            chars,
            len,
            index: 0,
        }
    }
}
pub fn s(value: &str) -> Parser {
    Parser::Str(Str::new(value))
}

impl Clone for Str {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            chars: self.chars.clone(),
            len: self.len,
            index: 0,
        }
    }
}

impl CharParser for Str {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else if ch.value == self.chars[self.index] {
            self.index += 1;
            if self.index == self.len {
                self.base.stat = Stat::Matched(ch.next_byte());
            }
        } else {
            self.base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}, current={}, byte_offset={}, stat={:?}",
        //     self.chars.iter().collect::<String>(),
        //     ch.value.escape_default(),
        //     ch.byte,
        //     self.base.stat
        // );
        self.base.stat
    }

    fn finish(&mut self, ch: &Char) -> Stat {
        if self.len == 0 {
            self.base.stat = Stat::Matched(ch.next_byte());
        } else {
            self.base.stat = Stat::Failed;
        };
        self.base.stat
    }

    fn reset(&mut self) {
        if !self.base.fresh {
            self.base.reset();
            self.index = 0;
        }
    }

    fn string(&self) -> String {
        format!("{}", self.chars.iter().collect::<String>())
    }
}
