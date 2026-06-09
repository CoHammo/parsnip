mod macros;
mod parsers;
mod tests;
mod types;

pub use parsers::*;
use types::*;

pub trait CharParser: Clone {
    fn take_char(&mut self, ch: &Char) -> Stat;
    fn finish(&mut self, ch: &Char) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}
