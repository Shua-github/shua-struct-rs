use crate::{BinaryField, Options};

impl<O, T> BinaryField<O> for Option<T>
where
    O: bitvec::prelude::BitOrder,
    T: BinaryField<O>,
{
    #[inline]
    fn parse(
        bits: &bitvec::prelude::BitSlice<u8, O>,
        opts: &Option<Options>,
    ) -> Result<Self, String> {
        let value = T::parse(bits, opts)?;
        Ok(Some(value))
    }

    #[inline]
    fn build(&self, opts: &Option<Options>) -> Result<bitvec::prelude::BitVec<u8, O>, String> {
        match self {
            Some(value) => value.build(opts),
            None => Ok(bitvec::prelude::BitVec::new()),
        }
    }

    #[inline]
    fn bit_len(&self, opts: &Option<Options>) -> usize {
        match self {
            Some(v) => v.bit_len(opts),
            None => 0,
        }
    }
}
