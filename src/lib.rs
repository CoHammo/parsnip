mod macros;
pub mod parsers;
mod testing;
pub mod types;

pub use parsers::*;
pub use types::*;

pub trait CharParser: Clone {
    fn take_char(&mut self, ch: &Char) -> Stat;
    fn finish(&mut self, ch: &Char) -> Stat;
    fn reset(&mut self);
    fn string(&self) -> String;
}
