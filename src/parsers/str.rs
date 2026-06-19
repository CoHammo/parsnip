use super::super::*;
use icu_normalizer::ComposingNormalizer;

parser!(Str s {
    value: &str => chars: Box<[char]>,
    => len: usize,
    char_index: usize = 0,
    match_byte: usize = 0,
} {
    chars = ComposingNormalizer::new_nfc().normalize(&value).chars().collect();
    len = chars.len();
});

impl Clone for Str {
    fn clone(&self) -> Self {
        Str::new(&self.chars.iter().collect::<String>())
    }
}

impl CharParser for Str {
    fn take_char(&mut self, ch: &Char) -> Stat {
        freshen!(self, ch);
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else {
            if ch.value == self.chars[self.char_index] {
                self.char_index += 1;
                if self.char_index == self.len {
                    self.base.stat = Stat::Matched(ch.byte);
                }
            } else {
                self.base.stat = Stat::Failed;
            }
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
            self.base.stat = Stat::Matched(ch.byte);
        } else {
            self.base.stat = Stat::Failed;
        };
        self.base.stat
    }

    fn reset(&mut self) {
        self.base.reset();
        self.char_index = 0;
    }

    fn string(&self) -> String {
        format!("{}", self.chars.iter().collect::<String>())
    }
}
