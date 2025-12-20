#[cfg(feature = "array")]
pub mod array;

#[cfg(feature = "bool")]
pub mod bool;

#[cfg(feature = "int")]
pub mod int;

#[cfg(feature = "float")]
pub mod float;

use bitvec::prelude::*;
use std::cell::Cell;

#[derive(Debug, Default)]
pub struct Options {
    pub name: String,
    pub size: usize,
    pub align: usize,
    pub sub_align: Cell<u8>,
}

impl Options {
    pub fn get_align(&self) -> Option<usize> {
        let n = self.sub_align.get();
        if n == 0 {
            return None;
        }
        let new = n - 1;
        self.sub_align.set(new);
        if new == 0 { Some(self.align) } else { None }
    }
}

pub trait BinaryField<O: BitOrder>: Sized {
    fn parse(bits: &BitSlice<u8, O>, opts: &Option<Options>) -> Result<(Self, usize), String>;

    fn build(&self, opts: &Option<Options>) -> Result<BitVec<u8, O>, String>;
}
