use crate::{BinaryField, Options};
use bitvec::prelude::*;
use std::ffi::CString;

impl BinaryField<Lsb0> for CString {
    #[inline]
    fn parse(bits: &BitSlice<u8, Lsb0>, _opts: &Option<Options>) -> Result<Self, String> {
        let mut bytes = Vec::new();
        let mut i = 0;

        while (i + 1) * 8 <= bits.len() {
            let byte = bits[i * 8..(i + 1) * 8].load_le::<u8>();
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            i += 1;
        }

        CString::new(bytes)
            .map_err(|_| "CString parse error: contains interior null byte".to_string())
    }

    #[inline]
    fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Lsb0>, String> {
        let mut bv = BitVec::<u8, Lsb0>::new();
        bv.extend_from_raw_slice(self.to_bytes_with_nul());
        Ok(bv)
    }

    #[inline]
    fn bit_len(&self, _opts: &Option<Options>) -> usize {
        self.to_bytes_with_nul().len() * 8
    }
}

impl BinaryField<Msb0> for CString {
    #[inline]
    fn parse(bits: &BitSlice<u8, Msb0>, _opts: &Option<Options>) -> Result<Self, String> {
        let mut bytes = Vec::new();
        let mut i = 0;

        while (i + 1) * 8 <= bits.len() {
            let byte = bits[i * 8..(i + 1) * 8].load_be::<u8>();
            if byte == 0 {
                break;
            }
            bytes.push(byte);
            i += 1;
        }

        CString::new(bytes)
            .map_err(|_| "CString parse error: contains interior null byte".to_string())
    }

    #[inline]
    fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Msb0>, String> {
        let mut bv = BitVec::<u8, Msb0>::new();
        bv.extend_from_raw_slice(self.to_bytes_with_nul());
        Ok(bv)
    }

    #[inline]
    fn bit_len(&self, _opts: &Option<Options>) -> usize {
        self.to_bytes_with_nul().len() * 8
    }
}
