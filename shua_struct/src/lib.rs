pub mod field;
pub use bitvec::prelude::*;
pub use shua_struct_macro::BinaryStruct;

#[derive(Debug, Default)]
pub struct Options {
    pub size: usize,
    pub align: usize,
    pub sub_align: std::cell::Cell<u8>,
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

pub trait BinaryField<O: bitvec::prelude::BitOrder>: Sized {
    fn parse(
        bits: &bitvec::prelude::BitSlice<u8, O>,
        opts: &Option<Options>,
    ) -> Result<(Self, usize), String>;

    fn build(&self, opts: &Option<Options>) -> Result<bitvec::prelude::BitVec<u8, O>, String>;
}
