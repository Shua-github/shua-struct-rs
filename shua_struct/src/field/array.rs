use crate::{Align, BinaryError, BinaryField, BitOrder, BitSlice, Count, ElemCtx};

#[inline]
fn align_offset(offset: usize, align: usize) -> usize {
    if align == 0 {
        offset
    } else if align.is_power_of_two() {
        (offset + align - 1) & !(align - 1)
    } else {
        (offset + align - 1) / align * align
    }
}

impl<T, O, Ctx, const N: usize> BinaryField<O, Ctx> for [T; N]
where
    O: BitOrder,
    T: BinaryField<O, <Ctx as ElemCtx>::ElemCtx> + Default + Copy,
    Ctx: ElemCtx + Align,
{
    type Error = BinaryError<T::Error, usize>;

    fn parse(bits: &BitSlice<u8, O>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let mut offset = 0;
        let mut arr = [T::default(); N];

        let align = ctx.get_align();
        let elem_ctx = ctx.get_elem_ctx();

        for (i, item) in arr.iter_mut().enumerate() {
            let v = T::parse(&bits[offset..], &elem_ctx).map_err(|e| Self::Error::At {
                index: i,
                source: e,
            })?;

            offset += v.bit_len(&elem_ctx);
            offset = align_offset(offset, align);
            *item = v;
        }

        Ok(arr)
    }

    fn build(&self, bits: &mut BitSlice<u8, O>, ctx: &Ctx) -> Result<(), Self::Error> {
        let mut offset = 0;

        let align = ctx.get_align();
        let elem_ctx = ctx.get_elem_ctx();

        for (i, item) in self.iter().enumerate() {
            let len = item.bit_len(&elem_ctx);

            item.build(&mut bits[offset..offset + len], &elem_ctx)
                .map_err(|e| Self::Error::At {
                    index: i,
                    source: e,
                })?;

            offset += len;
            offset = align_offset(offset, align);
        }

        Ok(())
    }

    fn bit_len(&self, ctx: &Ctx) -> usize {
        let mut total = 0;
        let align = ctx.get_align();
        let elem_ctx = ctx.get_elem_ctx();

        for item in self.iter() {
            total += item.bit_len(&elem_ctx);
            total = align_offset(total, align);
        }

        total
    }
}

impl<T, O: BitOrder, Ctx> BinaryField<O, Ctx> for Vec<T>
where
    O: BitOrder,
    T: BinaryField<O, <Ctx as ElemCtx>::ElemCtx>,
    Ctx: ElemCtx + Align + Count,
{
    type Error = BinaryError<T::Error, usize>;

    #[inline]
    fn parse(bits: &BitSlice<u8, O>, ctx: &Ctx) -> Result<Self, Self::Error> {
        let size = ctx.get_count();
        if size == 0 {
            return Ok(Vec::new());
        }

        let align = ctx.get_align();
        let elem_ctx = ctx.get_elem_ctx();

        let mut offset = 0;
        let mut vec = Vec::with_capacity(size);

        for i in 0..size {
            let item = T::parse(&bits[offset..], &elem_ctx).map_err(|e| Self::Error::At {
                index: i,
                source: e,
            })?;

            offset += item.bit_len(&elem_ctx);
            offset = align_offset(offset, align);
            vec.push(item);
        }

        Ok(vec)
    }

    #[inline]
    fn build(&self, bits: &mut BitSlice<u8, O>, ctx: &Ctx) -> Result<(), Self::Error> {
        let align = ctx.get_align();
        let mut offset = 0;
        let elem_ctx = ctx.get_elem_ctx();

        for (i, item) in self.iter().enumerate() {
            let len = item.bit_len(&elem_ctx);

            #[cfg(debug_assertions)]
            if offset + len > bits.len() {
                return Err(Self::Error::bit_count_mismatch(len, bits.len() - offset));
            }

            item.build(&mut bits[offset..offset + len], &elem_ctx)
                .map_err(|e| Self::Error::At {
                    index: i,
                    source: e,
                })?;

            offset += len;
            let prev_offset = offset;
            offset = align_offset(offset, align);

            #[cfg(debug_assertions)]
            if offset > bits.len() {
                let align_bytes = offset - prev_offset;
                return Err(Self::Error::bit_count_mismatch(
                    align_bytes,
                    bits.len() - prev_offset,
                ));
            }
        }

        Ok(())
    }

    #[inline]
    fn bit_len(&self, ctx: &Ctx) -> usize {
        let align = ctx.get_align();
        let mut total = 0;
        let elem_ctx = ctx.get_elem_ctx();

        for item in self.iter() {
            total += item.bit_len(&elem_ctx);
            total = align_offset(total, align);
        }

        total
    }
}
