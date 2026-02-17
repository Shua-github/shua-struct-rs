use crate::{BinaryField, Options};

impl<O, T> BinaryField<O> for Option<T>
where
    O: bitvec::prelude::BitOrder,
    T: BinaryField<O>,
{
    fn parse(
        bits: &bitvec::prelude::BitSlice<u8, O>,
        opts: &Option<Options>,
    ) -> Result<(Self, usize), String> {
        let (value, used) = T::parse(bits, opts)?;
        Ok((Some(value), used))
    }

    fn build(
        &self,
        opts: &Option<Options>,
    ) -> Result<bitvec::prelude::BitVec<u8, O>, String> {
        match self {
            Some(value) => value.build(opts),
            None => Ok(bitvec::prelude::BitVec::new()),
        }
    }
}
