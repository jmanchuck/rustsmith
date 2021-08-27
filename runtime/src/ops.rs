pub trait BitArithmetic {
    fn bit_and(self, other: Self) -> Self;
    fn bit_or(self, other: Self) -> Self;
    fn bit_xor(self, other: Self) -> Self;
    fn bit_neg(self) -> Self;

    fn bit_self_and(&mut self, other: Self);
    fn bit_self_or(&mut self, other: Self);
    fn bit_self_xor(&mut self, other: Self);
    fn bit_self_neg(&mut self);
}

macro_rules! impl_bit_arithmetic {
    (for $($t:ty),+) => {
        $(impl BitArithmetic for $t {

    #[inline]
    fn bit_and(self, other: Self) -> Self {
        self & other
    }

    #[inline]
    fn bit_or(self, other: Self) -> Self {
        self | other
    }

    #[inline]
    fn bit_xor(self, other: Self) -> Self {
        self ^ other
    }

    #[inline]
    fn bit_neg(self) -> Self {
        !self
    }

    #[inline]
    fn bit_self_and(&mut self, other: Self) {
        *self &= other;
    }

    #[inline]
    fn bit_self_or(&mut self, other: Self) {
        *self |= other
    }

    #[inline]
    fn bit_self_xor(&mut self, other: Self) {
        *self ^= other
    }

    #[inline]
    fn bit_self_neg(&mut self) {
        *self = !*self
    }

        })*
    };
}

impl_bit_arithmetic!(for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
