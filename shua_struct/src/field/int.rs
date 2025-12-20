use super::{BinaryField, Options};
use bitvec::prelude::*;

macro_rules! impl_bit_primitive {
    ($t:ty, $size_bits:expr) => {
        impl BinaryField<Lsb0> for $t {
            fn parse(
                bits: &BitSlice<u8, Lsb0>,
                _opts: &Option<Options>,
            ) -> Result<(Self, usize), String> {
                if bits.len() < $size_bits {
                    return Err(format!(
                        "{} parse error: not enough bits (needed {}, got {})",
                        stringify!($t),
                        $size_bits,
                        bits.len()
                    ));
                }
                let value = bits[0..$size_bits].load_le::<$t>();
                Ok((value, $size_bits))
            }

            fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Lsb0>, String> {
                let mut bv = BitVec::<u8, Lsb0>::new();
                let bytes = self.to_le_bytes();
                bv.extend_from_raw_slice(&bytes);
                bv.truncate($size_bits);
                Ok(bv)
            }
        }

        impl BinaryField<Msb0> for $t {
            fn parse(
                bits: &BitSlice<u8, Msb0>,
                _opts: &Option<Options>,
            ) -> Result<(Self, usize), String> {
                if bits.len() < $size_bits {
                    return Err(format!(
                        "{} parse error: not enough bits (needed {}, got {})",
                        stringify!($t),
                        $size_bits,
                        bits.len()
                    ));
                }
                let value = bits[0..$size_bits].load_be::<$t>();
                Ok((value, $size_bits))
            }

            fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Msb0>, String> {
                let mut bv = BitVec::<u8, Msb0>::new();
                let bytes = self.to_be_bytes();
                bv.extend_from_raw_slice(&bytes);
                bv.truncate($size_bits);
                Ok(bv)
            }
        }
    };
}
// uint
impl_bit_primitive!(u8, 8);
impl_bit_primitive!(u16, 16);
impl_bit_primitive!(u32, 32);
impl_bit_primitive!(u64, 64);
// int
impl_bit_primitive!(i8, 8);
impl_bit_primitive!(i16, 16);
impl_bit_primitive!(i32, 32);
impl_bit_primitive!(i64, 64);
