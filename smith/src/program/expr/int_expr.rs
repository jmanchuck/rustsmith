use super::expr::{ArithmeticExpr, Expr, LiteralExpr};
use crate::program::types::{IntTypeID, TypeID};

#[derive(Debug)]
pub enum IntValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl IntValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::I8(val) => val.to_string(),
            Self::I16(val) => val.to_string(),
            Self::I32(val) => val.to_string(),
            Self::I64(val) => val.to_string(),
            Self::I128(val) => val.to_string(),
            Self::U8(val) => val.to_string(),
            Self::U16(val) => val.to_string(),
            Self::U32(val) => val.to_string(),
            Self::U64(val) => val.to_string(),
            Self::U128(val) => val.to_string(),
        }
    }

    pub fn get_type(&self) -> TypeID {
        match self {
            Self::I8(_) => TypeID::IntType(IntTypeID::I8),
            Self::I16(_) => TypeID::IntType(IntTypeID::I16),
            Self::I32(_) => TypeID::IntType(IntTypeID::I32),
            Self::I64(_) => TypeID::IntType(IntTypeID::I64),
            Self::I128(_) => TypeID::IntType(IntTypeID::I128),
            Self::U8(_) => TypeID::IntType(IntTypeID::U8),
            Self::U16(_) => TypeID::IntType(IntTypeID::U16),
            Self::U32(_) => TypeID::IntType(IntTypeID::U32),
            Self::U64(_) => TypeID::IntType(IntTypeID::U64),
            Self::U128(_) => TypeID::IntType(IntTypeID::U128),
        }
    }
}

#[derive(Debug)]
pub struct IntExpr {
    value: IntValue,
}

impl IntExpr {
    pub fn get_type(&self) -> TypeID {
        self.value.get_type()
    }

    pub fn as_expr(self) -> Expr {
        Expr::Literal(LiteralExpr::Int(self))
    }

    pub fn as_arith_expr(self) -> ArithmeticExpr {
        ArithmeticExpr::Int(self)
    }

    pub fn new(value: IntValue) -> Self {
        IntExpr { value }
    }
    pub fn new_i8(value: i8) -> Self {
        IntExpr::new(IntValue::I8(value))
    }
    pub fn new_i16(value: i16) -> Self {
        IntExpr::new(IntValue::I16(value))
    }
    pub fn new_i32(value: i32) -> Self {
        IntExpr::new(IntValue::I32(value))
    }
    pub fn new_i64(value: i64) -> Self {
        IntExpr::new(IntValue::I64(value))
    }
    pub fn new_i128(value: i128) -> Self {
        IntExpr::new(IntValue::I128(value))
    }
    pub fn new_u8(value: u8) -> Self {
        IntExpr::new(IntValue::U8(value))
    }
    pub fn new_u16(value: u16) -> Self {
        IntExpr::new(IntValue::U16(value))
    }
    pub fn new_u32(value: u32) -> Self {
        IntExpr::new(IntValue::U32(value))
    }
    pub fn new_u64(value: u64) -> Self {
        IntExpr::new(IntValue::U64(value))
    }
    pub fn new_u128(value: u128) -> Self {
        IntExpr::new(IntValue::U128(value))
    }
}

impl ToString for IntExpr {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            self.value.to_string(),
            self.value.get_type().to_string()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn string_represents_correctly() {
        let value = IntValue::I32(5);
        assert_eq!(IntExpr::new(value).to_string(), "5");
    }
}
