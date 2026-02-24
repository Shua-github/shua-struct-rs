use crate::{BinaryError, BinaryField, BitField, BitSlice, Lsb0, Msb0};
use bitvec::mem::bits_of;

macro_rules! impl_bit_primitive {
    ($t:ty) => {
        impl BinaryField<Lsb0> for $t {
            type Error = BinaryError;
            #[inline]
            fn parse(bits: &BitSlice<u8, Lsb0>, _: &()) -> Result<Self, Self::Error> {
                let size_bits = bits_of::<$t>();
                if bits.len() < size_bits {
                    return Err(BinaryError::bit_count_mismatch(size_bits, bits.len()));
                }
                let value = bits[0..size_bits].load_le::<$t>();
                Ok(value)
            }

            #[inline]
            fn build(&self, bits: &mut BitSlice<u8, Lsb0>, _: &()) -> Result<(), Self::Error> {
                #[cfg(debug_assertions)]
                {
                    let size_bits = bits_of::<$t>();
                    if bits.len() != size_bits {
                        return Err(BinaryError::bit_count_mismatch(size_bits, bits.len()));
                    }
                }

                bits.store_le(*self);
                Ok(())
            }

            #[inline]
            fn bit_len(&self, _: &()) -> usize {
                bits_of::<$t>()
            }
        }

        impl BinaryField<Msb0> for $t {
            type Error = BinaryError;
            #[inline]
            fn parse(bits: &BitSlice<u8, Msb0>, _: &()) -> Result<Self, Self::Error> {
                let size_bits = bits_of::<$t>();
                if bits.len() < size_bits {
                    return Err(BinaryError::bit_count_mismatch(size_bits, bits.len()));
                }
                let value = bits[0..size_bits].load_be::<$t>();
                Ok(value)
            }

            #[inline]
            fn build(&self, bits: &mut BitSlice<u8, Msb0>, _: &()) -> Result<(), Self::Error> {
                #[cfg(debug_assertions)]
                {
                    let size_bits = bits_of::<$t>();
                    if bits.len() != size_bits {
                        return Err(BinaryError::bit_count_mismatch(size_bits, bits.len()));
                    }
                }

                bits.store_be(*self);
                Ok(())
            }

            #[inline]
            fn bit_len(&self, _: &()) -> usize {
                bits_of::<$t>()
            }
        }
    };
}

// uint
impl_bit_primitive!(u8);
impl_bit_primitive!(u16);
impl_bit_primitive!(u32);
impl_bit_primitive!(u64);

// int
impl_bit_primitive!(i8);
impl_bit_primitive!(i16);
impl_bit_primitive!(i32);
impl_bit_primitive!(i64);
