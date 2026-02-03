use crate::{BinaryField, Options};
use bitvec::prelude::*;
use std::ffi::CString;

impl BinaryField<Lsb0> for CString {
    fn parse(bits: &BitSlice<u8, Lsb0>, _opts: &Option<Options>) -> Result<(Self, usize), String> {
        let mut bytes = vec![];
        let mut i = 0;
        while i * 8 < bits.len() {
            let byte = bits[i * 8..(i + 1) * 8].load_le::<u8>();
            if byte == 0 {
                i += 1;
                break;
            }
            bytes.push(byte);
            i += 1;
        }

        match CString::new(bytes) {
            Ok(s) => Ok((s, i * 8)),
            Err(_) => Err("CString parse error: contains interior null byte".to_string()),
        }
    }

    fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Lsb0>, String> {
        let mut bv = BitVec::<u8, Lsb0>::new();
        bv.extend_from_raw_slice(self.to_bytes_with_nul());
        Ok(bv)
    }
}

impl BinaryField<Msb0> for CString {
    fn parse(bits: &BitSlice<u8, Msb0>, _opts: &Option<Options>) -> Result<(Self, usize), String> {
        let mut bytes = vec![];
        let mut i = 0;
        while i * 8 < bits.len() {
            let byte = bits[i * 8..(i + 1) * 8].load_be::<u8>();
            if byte == 0 {
                i += 1;
                break;
            }
            bytes.push(byte);
            i += 1;
        }

        match CString::new(bytes) {
            Ok(s) => Ok((s, i * 8)),
            Err(_) => Err("CString parse error: contains interior null byte".to_string()),
        }
    }

    fn build(&self, _opts: &Option<Options>) -> Result<BitVec<u8, Msb0>, String> {
        let mut bv = BitVec::<u8, Msb0>::new();
        bv.extend_from_raw_slice(self.to_bytes_with_nul());
        Ok(bv)
    }
}
