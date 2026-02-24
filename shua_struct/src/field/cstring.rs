use crate::{BinaryError, BinaryField, BitField, BitSlice, Lsb0, Msb0};
use std::ffi::{CString, NulError};

impl BinaryField<Lsb0> for CString {
    type Error = BinaryError<(), (), NulError>;

    #[inline]
    fn parse(bits: &BitSlice<u8, Lsb0>, _: &()) -> Result<Self, Self::Error> {
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

        CString::new(bytes).map_err(Self::Error::Custom)
    }

    #[inline]
    fn build(&self, bits: &mut BitSlice<u8, Lsb0>, _: &()) -> Result<(), Self::Error> {
        let bytes = self.to_bytes_with_nul();
        #[cfg(debug_assertions)]
        {
            let data_len = bytes.len() * 8;
            let input_len = bits.len();
            if input_len < data_len {
                return Err(Self::Error::bit_count_mismatch(data_len, input_len));
            }
        }

        for (i, &b) in bytes.iter().enumerate() {
            bits[i * 8..(i + 1) * 8].store_le(b);
        }

        Ok(())
    }

    #[inline]
    fn bit_len(&self, _: &()) -> usize {
        self.to_bytes_with_nul().len() * 8
    }
}

impl BinaryField<Msb0> for CString {
    type Error = BinaryError<(), (), NulError>;

    #[inline]
    fn parse(bits: &BitSlice<u8, Msb0>, _: &()) -> Result<Self, Self::Error> {
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

        CString::new(bytes).map_err(Self::Error::Custom)
    }

    #[inline]
    fn build(&self, bits: &mut BitSlice<u8, Msb0>, _: &()) -> Result<(), Self::Error> {
        let bytes = self.to_bytes_with_nul();
        #[cfg(debug_assertions)]
        {
            let data_len = bytes.len() * 8;
            let input_len = bits.len();
            if input_len < data_len {
                return Err(Self::Error::bit_count_mismatch(data_len, input_len));
            }
        }

        for (i, &b) in bytes.iter().enumerate() {
            bits[i * 8..(i + 1) * 8].store_be(b);
        }

        Ok(())
    }

    #[inline]
    fn bit_len(&self, _: &()) -> usize {
        self.to_bytes_with_nul().len() * 8
    }
}
