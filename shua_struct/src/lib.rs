pub mod field;

use std::fmt::{self, Debug};

#[doc(no_inline)]
pub use bitvec::field::BitField;
#[doc(no_inline)]
pub use bitvec::prelude::*;
pub use shua_struct_macro::BinaryField;

pub trait BinaryField<O: BitOrder, Ctx = ()>: Sized {
    type Error;

    fn parse(bits: &BitSlice<u8, O>, ctx: &Ctx) -> Result<Self, Self::Error>;

    fn build(&self, bits: &mut BitSlice<u8, O>, ctx: &Ctx) -> Result<(), Self::Error>;

    fn bit_len(&self, ctx: &Ctx) -> usize;

    #[inline]
    fn to_bitvec(&self, ctx: &Ctx) -> Result<BitVec<u8, O>, Self::Error> {
        let bit_len = self.bit_len(ctx);
        let mut bv: BitVec<u8, O> = BitVec::repeat(false, bit_len);
        self.build(&mut bv, ctx)?;
        Ok(bv)
    }
}

pub trait Count {
    fn get_count(&self) -> usize;
}

pub trait Align {
    fn get_align(&self) -> usize;
}

pub trait ElemCtx {
    type ElemCtx;
    fn get_elem_ctx(&self) -> Self::ElemCtx;
}

// Default implementations for empty context
impl Align for () {
    #[inline]
    fn get_align(&self) -> usize {
        0
    }
}

impl ElemCtx for () {
    type ElemCtx = ();

    #[inline]
    fn get_elem_ctx(&self) -> Self::ElemCtx {}
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryError<S = (), I = (), C = ()> {
    At { index: I, source: S },
    BitCountMismatch { needed: usize, got: usize },
    Custom(C),
}

impl<S, I, C> BinaryError<S, I, C> {
    #[inline]
    pub fn bit_count_mismatch(needed: usize, got: usize) -> Self {
        BinaryError::BitCountMismatch { needed, got }
    }
}

impl<S: fmt::Debug, I: fmt::Debug, C: fmt::Debug> fmt::Display for BinaryError<S, I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryError::At { index, source } => {
                write!(f, "Error at index {:?}: {:?}", index, source)
            }
            BinaryError::BitCountMismatch { needed, got } => {
                write!(f, "Bit count mismatch: needed {}, got {}", needed, got)
            }
            BinaryError::Custom(c) => write!(f, "Custom error: {:?}", c),
        }
    }
}
