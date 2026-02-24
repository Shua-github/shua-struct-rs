use crate::{BinaryField, BitOrder, BitSlice};

impl<O, T, Ctx> BinaryField<O, Ctx> for Option<T>
where
    O: BitOrder,
    T: BinaryField<O, Ctx>,
{
    type Error = T::Error;
    #[inline]
    fn parse(bits: &BitSlice<u8, O>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let value = T::parse(bits, ctx)?;
        Ok(Some(value))
    }

    #[inline]
    fn build(&self, bits: &mut BitSlice<u8, O>, ctx: &Ctx) -> Result<(), Self::Error> {
        match self {
            Some(value) => value.build(bits, ctx),
            None => Ok(()),
        }
    }

    #[inline]
    fn bit_len(&self, ctx: &Ctx) -> usize {
        match self {
            Some(v) => v.bit_len(ctx),
            None => 0,
        }
    }
}
