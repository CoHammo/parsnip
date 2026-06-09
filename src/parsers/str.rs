use super::super::*;
use unicode_segmentation::UnicodeSegmentation;

parser!(Str s {
    value: &str => val: String,
    => chars: Vec<Box<str>>,
    => len: usize,
    char_index: usize = 0,
} {
    val = value.to_string();
    chars = value
        .graphemes(true)
        .map(|c| c.to_string().into_boxed_str())
        .collect();
    len = chars.len();
});

impl Clone for Str {
    fn clone(&self) -> Self {
        Str::new(&self.val)
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
        // println!(
        //     "matching={:?} current={}, byte_offset={}, stat={:?}",
        //     self.chars,
        //     ch.value.escape_default(),
        //     ch.byte_offset,
        //     self.stat
        // );
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
        format!("{}", self.val.escape_default())
    }
}
