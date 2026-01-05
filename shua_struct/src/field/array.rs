use crate::{BinaryField, Options};
use bitvec::prelude::*;

impl<T, O: BitOrder, const N: usize> BinaryField<O> for [T; N]
where
    T: BinaryField<O> + Default + Copy,
{
    fn parse(bits: &BitSlice<u8, O>, raw_opts: &Option<Options>) -> Result<(Self, usize), String> {
        let align = raw_opts.as_ref().and_then(|opts| opts.get_align());

        let mut offset = 0;
        let mut arr: [T; N] = [T::default(); N];

        for i in 0..N {
            let (v, l) = T::parse(&bits[offset..], raw_opts)?;
            offset += l;
            if let Some(align) = align {
                let r = offset % align;
                if r != 0 {
                    offset += align - r;
                }
            }
            arr[i] = v;
        }

        Ok((arr, offset))
    }

    fn build(&self, raw_opts: &Option<Options>) -> Result<BitVec<u8, O>, String> {
        let align = raw_opts.as_ref().and_then(|opts| opts.get_align());

        let mut bv = BitVec::<u8, O>::new();
        for item in self.iter() {
            if let Some(align) = align {
                let r = bv.len() % align;
                if r != 0 {
                    bv.resize(bv.len() + (align - r), false);
                }
            }
            bv.extend(item.build(raw_opts)?);
        }
        Ok(bv)
    }
}

impl<T, O: BitOrder> BinaryField<O> for Vec<T>
where
    T: BinaryField<O> + Default,
{
    fn parse(bits: &BitSlice<u8, O>, raw_opts: &Option<Options>) -> Result<(Self, usize), String> {
        let opts = raw_opts.as_ref().ok_or("Vec parse error: missing opts")?;
        if opts.size == 0 {
            return Err("Vec parse error: missing size".to_string());
        }
        let align = opts.get_align();

        let mut vec = Vec::with_capacity(opts.size);
        let mut offset = 0;

        for _ in 0..opts.size {
            let (item, l) = T::parse(&bits[offset..], raw_opts)?;
            offset += l;
            if let Some(align) = align {
                let r = offset % align;
                if r != 0 {
                    offset += align - r;
                }
            }
            vec.push(item);
        }
        Ok((vec, offset))
    }

    fn build(&self, raw_opts: &Option<Options>) -> Result<BitVec<u8, O>, String> {
        let opts = raw_opts.as_ref().ok_or("Vec build error: missing opts")?;
        let align = opts.get_align();

        let mut bv = BitVec::<u8, O>::new();
        for item in self.iter() {
            bv.extend(item.build(raw_opts)?);
            if let Some(align) = align {
                let r = bv.len() % align;
                if r != 0 {
                    bv.resize(bv.len() + (align - r), false);
                }
            }
        }
        Ok(bv)
    }
}
