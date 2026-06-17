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
    fn take_char(&mut self, chars: &mut ParseChars) -> Stat {
        // if self.base.fresh {
        //     self.base.start_byte = ch.byte;
        //     self.base.fresh = false;
        //     if self.len == 0 {
        //         self.base.stat = Stat::Failed;
        //         return self.base.stat;
        //     } else {
        //         let mut offset = 0;
        //         let mut prev_start_byte = 0;
        //         let mut my_chars = self.chars.iter();
        //         while let Some(c) = chars.peek(offset)
        //             && let Some(my_c) = my_chars.next()
        //         {
        //             if c.value == *my_c {
        //                 prev_start_byte = c.byte;
        //                 offset += 1;
        //             } else {
        //                 self.base.stat = Stat::Failed;
        //                 return self.base.stat;
        //             }
        //         }
        //         if offset == self.len {
        //             self.match_byte = prev_start_byte;
        //         }
        //     }
        // }
        // if ch.byte == self.match_byte {
        //     self.base.stat = Stat::Matched(ch.next_byte());
        // }

        freshen!(self, chars.char);
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else {
            if chars.char.value == self.chars[self.char_index] {
                self.char_index += 1;
                if self.char_index == self.len {
                    self.base.stat = Stat::Matched(chars.char.byte);
                }
            } else {
                self.base.stat = Stat::Failed;
            }
        }
        // println!(
        //     "matching={:?} current={}, byte_offset={}, stat={:?}",
        //     self.chars,
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
        self.base.reset();
        self.char_index = 0;
    }

    fn string(&self) -> String {
        format!("{}", self.chars.iter().collect::<String>())
    }
}
