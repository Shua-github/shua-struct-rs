use crate::{BinaryError, BinaryField, BitOrder, BitSlice};

impl<O: BitOrder, Ctx> BinaryField<O, Ctx> for bool {
    type Error = BinaryError;

    #[inline]
    fn parse(bits: &BitSlice<u8, O>, _: &Ctx) -> Result<Self, Self::Error> {
        if bits.is_empty() {
            return Err(BinaryError::bit_count_mismatch(1, bits.len()));
        }
        Ok(bits[0])
    }

    #[inline]
    fn build(&self, bits: &mut BitSlice<u8, O>, _: &Ctx) -> Result<(), Self::Error> {
        #[cfg(debug_assertions)]
        if bits.is_empty() {
            return Err(BinaryError::bit_count_mismatch(1, bits.len()));
        }
        bits.set(0, *self);
        Ok(())
    }

    #[inline]
    fn bit_len(&self, _: &Ctx) -> usize {
        1
    }
}
