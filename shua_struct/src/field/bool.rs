use super::{BinaryField, Options};
use bitvec::prelude::*;

impl<O: BitOrder> BinaryField<O> for bool {
    fn parse(bits: &BitSlice<u8, O>, _opts: &Option<Options>) -> Result<(Self, usize), String> {
        if bits.len() < 1 {
            return Err("bool parse error: not enough bits".to_string());
        }
        Ok((bits[0], 1))
    }

    fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, O>, String> {
        let mut bv = BitVec::<u8, O>::new();
        bv.push(*self);
        Ok(bv)
    }
}
