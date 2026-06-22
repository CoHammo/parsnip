use icu_normalizer::ComposingNormalizer;

use super::super::*;

// #[derive(Debug)]
// pub struct Str {
//     pub base: BaseParser,
//     chars: Box<[char]>,
//     len: usize,
//     index: usize,
// }
// impl Str {
//     pub fn new(value: &str) -> Self {
//         let chars = ComposingNormalizer::new_nfc()
//             .normalize(value)
//             .chars()
//             .collect::<Box<[char]>>();
//         let len = chars.len();
//         Self {
//             base: BaseParser::new(),
//             chars,
//             len,
//             index: 0,
//         }
//     }
// }
// pub fn s(value: &str) -> Parser {
//     Parser::Str(Str::new(value))
// }

// impl Clone for Str {
//     fn clone(&self) -> Self {
//         Self {
//             base: BaseParser::new(),
//             chars: self.chars.clone(),
//             len: self.len,
//             index: 0,
//         }
//     }
// }

// impl CharParser for Str {
//     fn take_char(&mut self, ch: &Char) -> Stat {
//         // freshen!(self, ch);
//         if self.len == 0 {
//             self.base.stat = Stat::Failed;
//         } else if ch.value == self.chars[self.index] {
//             self.index += 1;
//             if self.index == self.len {
//                 self.base.stat = Stat::Matched(ch.next_byte());
//             }
//         } else {
//             self.base.stat = Stat::Failed;
//         }
//         // println!(
//         //     "matching={:?}, current={}, byte_offset={}, stat={:?}",
//         //     self.chars.iter().collect::<String>(),
//         //     ch.value.escape_default(),
//         //     ch.byte,
//         //     self.base.stat
//         // );
//         self.base.stat
//     }

//     fn finish(&mut self, ch: &Char) -> Stat {
//         if self.len == 0 {
//             self.base.stat = Stat::Matched(ch.next_byte());
//         } else {
//             self.base.stat = Stat::Failed;
//         };
//         self.base.stat
//     }

//     fn reset(&mut self) {
//         if !self.base.fresh {
//             self.base.reset();
//             self.index = 0;
//         }
//     }

//     fn string(&self) -> String {
//         format!("{}", self.chars.iter().collect::<String>())
//     }
// }

#[derive(Debug)]
pub struct It<T: PI> {
    pub base: BaseParser,
    items: Box<[ParseItem<T>]>,
    len: usize,
    index: usize,
}
impl<T: PI> It<T> {
    pub fn new(value: impl ToParseItems<T>) -> Self {
        let items = value.to_items();
        let len = items.len();
        Self {
            base: BaseParser::new(),
            items,
            len,
            index: 0,
        }
    }
}

impl<T: PI> Clone for It<T> {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            items: self.items.clone(),
            len: self.len,
            index: 0,
        }
    }
}

impl<T: PI> ItemParser<T> for It<T> {
    fn take(&mut self, item: &IterItem<T>) -> Stat {
        if self.base.fresh {
            self.base.start = item.index();
            self.base.fresh = false;
        }
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else if item.value == self.items[self.index].0 {
            self.index += 1;
            if self.index == self.len {
                self.base.stat = Stat::Matched(item.index() + 1);
            }
        } else {
            self.base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}({}), byte={}, index={}, stat={:?}",
        //     self.bytes,
        //     self.bytes[self.index - 1],
        //     byte.value,
        //     byte.index(),
        //     self.base.stat
        // );
        self.base.stat
    }

    fn finish(&mut self, item: &IterItem<T>) -> Stat {
        if self.len == 0 {
            self.base.stat = Stat::Matched(item.index() + 1);
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
        format!("Its({:?})", self.items)
    }
}
pub fn it<T: PI>(value: impl ToParseItems<T>) -> Parser<T> {
    Parser::It(It::new(value))
}

#[derive(Debug)]
pub struct Bytes {
    pub base: BaseParser,
    bytes: Box<[u8]>,
    len: usize,
    index: usize,
}
impl Bytes {
    pub fn new(value: &str) -> Self {
        let norm = ComposingNormalizer::new_nfc().normalize(value);
        let len = norm.len();
        Self {
            base: BaseParser::new(),
            bytes: norm.as_bytes().into(),
            len,
            index: 0,
        }
    }
}

impl Clone for Bytes {
    fn clone(&self) -> Self {
        Self {
            base: BaseParser::new(),
            bytes: self.bytes.clone(),
            len: self.len,
            index: 0,
        }
    }
}

impl ByteParser for Bytes {
    fn take(&mut self, byte: &Byte) -> Stat {
        if self.base.fresh {
            self.base.start = byte.index();
            self.base.fresh = false;
        }
        if self.len == 0 {
            self.base.stat = Stat::Failed;
        } else if byte.value == self.bytes[self.index] {
            self.index += 1;
            if self.index == self.len {
                self.base.stat = Stat::Matched(byte.index() + 1);
            }
        } else {
            self.base.stat = Stat::Failed;
        }
        // println!(
        //     "matching={:?}({}), byte={}, index={}, stat={:?}",
        //     self.bytes,
        //     self.bytes[self.index - 1],
        //     byte.value,
        //     byte.index(),
        //     self.base.stat
        // );
        self.base.stat
    }

    fn finish(&mut self, byte: &Byte) -> Stat {
        if self.len == 0 {
            self.base.stat = Stat::Matched(byte.index() + 1);
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
        format!("Bytes({:?})", self.bytes)
    }
}
