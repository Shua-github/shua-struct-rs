use super::{BinaryField, Options};
use bitvec::prelude::*;

macro_rules! impl_bit_float {
    ($t:ty, $int:ty, $size_bits:expr) => {
        impl BinaryField<Lsb0> for $t {
            fn parse(bits: &BitSlice<u8, Lsb0>, _opts: &Option<Options>) -> Result<(Self, usize), String> {
                if bits.len() < $size_bits {
                    return Err(format!(
                        "{} parse error: not enough bits (needed {}, got {})",
                        stringify!($t),
                        $size_bits,
                        bits.len()
                    ));
                }
                let raw_bits = bits[0..$size_bits].load_le::<$int>();
                Ok((<$t>::from_bits(raw_bits), $size_bits))
            }

            fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Lsb0>, String> {
                let mut bv = BitVec::<u8, Lsb0>::new();
                let bytes = self.to_bits().to_le_bytes();
                bv.extend_from_raw_slice(&bytes);
                bv.truncate($size_bits);
                Ok(bv)
            }
        }

        impl BinaryField<Msb0> for $t {
            fn parse(bits: &BitSlice<u8, Msb0>, _opts: &Option<Options>) -> Result<(Self, usize), String> {
                if bits.len() < $size_bits {
                    return Err(format!(
                        "{} parse error: not enough bits (needed {}, got {})",
                        stringify!($t),
                        $size_bits,
                        bits.len()
                    ));
                }
                let raw_bits = bits[0..$size_bits].load_be::<$int>();
                Ok((<$t>::from_bits(raw_bits), $size_bits))
            }

            fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Msb0>, String> {
                let mut bv = BitVec::<u8, Msb0>::new();
                let bytes = self.to_bits().to_be_bytes();
                bv.extend_from_raw_slice(&bytes);
                bv.truncate($size_bits);
                Ok(bv)
            }
        }
    };
}

impl_bit_float!(f32, u32, 32);
impl_bit_float!(f64, u64, 64);