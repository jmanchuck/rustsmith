pub trait SafeArithmetic {
    fn safe_add(self, other: Self) -> Self;
    fn safe_sub(self, other: Self) -> Self;
    fn safe_mul(self, other: Self) -> Self;
    fn safe_div(self, other: Self) -> Self;
    fn safe_modulo(self, other: Self) -> Self;
    fn safe_self_add(&mut self, other: Self);
    fn safe_self_sub(&mut self, other: Self);
    fn safe_self_mul(&mut self, other: Self);
    fn safe_self_div(&mut self, other: Self);
    fn safe_self_modulo(&mut self, other: Self);
}

macro_rules! impl_safe_arithmetic {
    (for $($t:ty),+) => {
        $(impl SafeArithmetic for $t {

    #[inline]
    fn safe_add(self, other: Self) -> Self {
        let checked_result = self.checked_add(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    #[inline]
    fn safe_sub(self, other: Self) -> Self {
        let checked_result = self.checked_sub(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    #[inline]
    fn safe_mul(self, other: Self) -> Self {
        let checked_result = self.checked_mul(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    #[inline]
    fn safe_div(self, other: Self) -> Self {
        let checked_result = self.checked_div(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    #[inline]
    fn safe_modulo(self, other: Self) -> Self {
        let checked_result = self.checked_rem(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    #[inline]
    fn safe_self_add(&mut self, other: Self) {
        *self = self.safe_add(other);
    }

    #[inline]
    fn safe_self_sub(&mut self, other: Self) {
        *self = self.safe_sub(other);
    }

    #[inline]
    fn safe_self_mul(&mut self, other: Self) {
        *self = self.safe_mul(other);
    }

    #[inline]
    fn safe_self_div(&mut self, other: Self) {
        *self = self.safe_div(other);
    }

    #[inline]
    fn safe_self_modulo(&mut self, other: Self) {
        *self = self.safe_modulo(other);
    }

        })*
    };
}

impl_safe_arithmetic!(for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
