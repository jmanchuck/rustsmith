use super::{expr::Expr, func_call_expr::FunctionCallExpr};
use crate::program::{
    types::{IntTypeID, TypeID},
    var::Var,
};

pub enum ArithmeticExpr {
    Int(IntExpr),
    Binary(Box<BinaryExpr>),
    Var(Var),
    Func(FunctionCallExpr),
}

impl ArithmeticExpr {
    pub fn new_from_bin_expr(expr: BinaryExpr) -> Self {
        ArithmeticExpr::Binary(Box::new(expr))
    }

    pub fn new_from_int_expr(expr: IntExpr) -> Self {
        ArithmeticExpr::Int(expr)
    }

    pub fn as_expr(self) -> Expr {
        Expr::Arithmetic(self)
    }

    pub fn get_type(&self) -> TypeID {
        match self {
            Self::Int(s) => s.get_type(),
            Self::Binary(s) => s.get_type(),
            Self::Var(s) => s.get_type(),
            Self::Func(s) => s.get_type(),
        }
    }
}

impl ToString for ArithmeticExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Int(s) => s.to_string(),
            Self::Binary(s) => (*s).to_string_safe(),
            Self::Var(s) => s.to_string(),
            Self::Func(s) => s.to_string(),
        }
    }
}

impl From<Expr> for ArithmeticExpr {
    fn from(expr: Expr) -> Self {
        if let Expr::Arithmetic(result) = expr {
            result
        } else {
            panic!("Could not perform conversion from Expr to ArithmeticExpr")
        }
    }
}

pub struct BinaryExpr {
    left: ArithmeticExpr,
    right: ArithmeticExpr,
    op: BinaryOp,
}

impl BinaryExpr {
    pub fn new(left: ArithmeticExpr, right: ArithmeticExpr, op: BinaryOp) -> Self {
        BinaryExpr { left, right, op }
    }

    pub fn as_arith_expr(self) -> ArithmeticExpr {
        ArithmeticExpr::Binary(Box::new(self))
    }

    pub fn get_type(&self) -> TypeID {
        self.left.get_type()
    }

    pub fn to_string_safe(&self) -> String {
        // This has the form of 5.checked_add(6u8) where 5 and 6 are literal u8 expressions
        // The annotation is only required when a literal is provided as argument
        // The checked arithmetic operations require type annotations in the argument
        format!(
            "{}.{}({})",
            self.left.to_string(),
            self.op.to_string_safe(),
            self.right.to_string(),
        )
    }
}

impl ToString for BinaryExpr {
    fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.left.to_string(),
            self.op.to_string(),
            self.right.to_string()
        )
    }
}

pub enum BinaryOp {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
}

impl BinaryOp {
    pub fn to_string_safe(&self) -> String {
        match self {
            BinaryOp::ADD => String::from("safe_add"),
            BinaryOp::SUB => String::from("safe_sub"),
            BinaryOp::MUL => String::from("safe_mul"),
            BinaryOp::DIV => String::from("safe_div"),
            BinaryOp::MOD => String::from("safe_modulo"),
        }
    }

    pub fn to_string_self_safe(&self) -> String {
        match self {
            BinaryOp::ADD => String::from("safe_self_add"),
            BinaryOp::SUB => String::from("safe_self_sub"),
            BinaryOp::MUL => String::from("safe_self_mul"),
            BinaryOp::DIV => String::from("safe_self_div"),
            BinaryOp::MOD => String::from("safe_self_modulo"),
        }
    }
}

impl ToString for BinaryOp {
    fn to_string(&self) -> String {
        match self {
            Self::ADD => String::from("+"),
            Self::SUB => String::from("-"),
            Self::MUL => String::from("*"),
            Self::DIV => String::from("/"),
            Self::MOD => String::from("%"),
        }
    }
}

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
        ArithmeticExpr::new_from_int_expr(self).as_expr()
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
    fn binary_expr_has_correct_string_representation() {
        let left_expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(30));
        let right_expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(5));

        // binary expression 30 + 5
        let binary_expr = BinaryExpr::new(left_expr, right_expr, BinaryOp::ADD);

        assert_eq!(binary_expr.to_string(), "30i32 + 5i32");
    }

    #[test]
    fn int_expr_has_correct_string_representation() {
        let value = IntValue::I32(5);
        assert_eq!(IntExpr::new(value).to_string(), "5i32");
    }
}
