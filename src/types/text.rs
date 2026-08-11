use std::{ops::RangeBounds, str::Bytes};

use super::*;
use icu_normalizer::ComposingNormalizer;

pub struct Text {
    value: String,
}
impl Text {
    pub fn new(value: String) -> Self {
        Self {
            value: ComposingNormalizer::new_nfc()
                .normalize(&value)
                .into_owned(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Parses<u8> for Text {
    type Iter<'a>
        = Bytes<'a>
    where
        Self: 'a;

    fn to_parse_iter(&self, range: impl RangeBounds<usize>) -> ParseIter<u8, Bytes<'_>> {
        ParseIter::new(self.value.bytes(), self.value.len(), range)
    }

    fn to_inner_store(&self) -> Box<[u8]> {
        self.value.as_bytes().into()
    }
}
