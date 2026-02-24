use crate::{BinaryError, BinaryField, BitField, BitSlice, Lsb0, Msb0};
use bitvec::mem::bits_of;

macro_rules! impl_bit_float {
    ($t:ty, $int:ty) => {
        impl BinaryField<Lsb0> for $t {
            type Error = BinaryError;
            #[inline]
            fn parse(bits: &BitSlice<u8, Lsb0>, _: &()) -> Result<Self, Self::Error> {
                let size_bits = bits_of::<$t>();
                if bits.len() < size_bits {
                    return Err(BinaryError::bit_count_mismatch(size_bits, bits.len()));
                }

                let raw_bits = bits[0..size_bits].load_le::<$int>();
                Ok(<$t>::from_bits(raw_bits))
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

                let raw_bits: $int = self.to_bits();
                bits.store_le(raw_bits);
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

                let raw_bits = bits[0..size_bits].load_be::<$int>();
                Ok(<$t>::from_bits(raw_bits))
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

                let raw_bits: $int = self.to_bits();
                bits.store_be(raw_bits);
                Ok(())
            }

            #[inline]
            fn bit_len(&self, _: &()) -> usize {
                bits_of::<$t>()
            }
        }
    };
}

impl_bit_float!(f32, u32);
impl_bit_float!(f64, u64);
