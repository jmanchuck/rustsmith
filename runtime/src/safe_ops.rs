pub trait SafeArithmetic {
    fn safe_add(self, other: Self) -> Self;
    fn safe_sub(self, other: Self) -> Self;
    fn safe_mul(self, other: Self) -> Self;
    fn safe_div(self, other: Self) -> Self;
}

macro_rules! impl_safe_arithmetic {
    (for $($t:ty),+) => {
        $(impl SafeArithmetic for $t {
            fn safe_add(self, other: Self) -> Self {
        let checked_result = self.checked_add(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    fn safe_sub(self, other: Self) -> Self {
        let checked_result = self.checked_sub(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    fn safe_mul(self, other: Self) -> Self {
        let checked_result = self.checked_mul(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }

    fn safe_div(self, other: Self) -> Self {
        let checked_result = self.checked_div(other);
        match checked_result {
            Some(result) => result,
            None => self,
        }
    }
        })*
    };
}

impl_safe_arithmetic!(for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
