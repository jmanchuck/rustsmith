use super::{
    array_expr::ArrayIndexExpr, bool_expr::BoolExpr, expr::Expr, func_call_expr::FunctionCallExpr,
};
use crate::program::{
    types::{IntTypeID, TypeID},
    var::Var,
};
pub enum ArithmeticExpr {
    Int(IntExpr),
    Binary(Box<BinaryExpr>),
    Cast(Box<CastExpr>),
    Unary(Box<UnaryExpr>),
    ArrayIndex(Box<ArrayIndexExpr>),
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
            Self::Cast(s) => s.get_type(),
            Self::Unary(s) => s.get_type(),
            Self::ArrayIndex(s) => s.get_type(),
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
            Self::Cast(s) => (*s).to_string(),
            Self::Unary(s) => (*s).to_string_safe(),
            Self::ArrayIndex(s) => (*s).to_string(),
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

impl From<FunctionCallExpr> for ArithmeticExpr {
    fn from(expr: FunctionCallExpr) -> Self {
        ArithmeticExpr::Func(expr)
    }
}

impl From<Var> for ArithmeticExpr {
    fn from(expr: Var) -> Self {
        ArithmeticExpr::Var(expr)
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
        // The receiver is always parenthesized: a negative literal receiver
        // like -5i8.wrapping_add(x) would otherwise parse as
        // -(5i8.wrapping_add(x)).
        //
        // Division and remainder bind both operands to locals before the
        // checked call because operands may contain function calls with &mut
        // side effects, so they must be evaluated exactly once. The fallback
        // value on None (division by zero, or MIN / -1) is the left operand,
        // which is deterministic.
        let left = self.left.to_string();
        let right = self.right.to_string();
        match self.op {
            BinaryOp::BITAND | BinaryOp::BITOR | BinaryOp::BITXOR => {
                format!("({} {} {})", left, self.op.to_string(), right)
            }
            BinaryOp::DIV | BinaryOp::MOD => {
                let method = if let BinaryOp::DIV = self.op {
                    "checked_div"
                } else {
                    "checked_rem"
                };
                // Parenthesized because a bare block in expression-with-block
                // positions (for-loop headers, if conditions) would be parsed
                // as the loop/if body instead of an operand.
                format!(
                    "({{ let lhs = {}; let rhs = {}; lhs.{}(rhs).unwrap_or(lhs) }})",
                    left, right, method
                )
            }
            BinaryOp::SHL | BinaryOp::SHR => {
                // wrapping_shl/shr because plain << / >> with an amount >=
                // bit-width panics under overflow-checks=on and masks when
                // off (a false divergence across fuzz configs); wrapping_*
                // masks the amount identically in every config. The amount
                // operand may be any int type, so it is cast to u32.
                format!(
                    "({}).{}(({}) as u32)",
                    left,
                    self.op.to_string_safe(),
                    right
                )
            }
            _ => format!("({}).{}({})", left, self.op.to_string_safe(), right),
        }
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

#[derive(Copy, Clone)]
pub enum BinaryOp {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    BITAND,
    BITOR,
    BITXOR,
    SHL,
    SHR,
}

impl BinaryOp {
    pub const ALL: &'static [Self] = &[
        Self::ADD,
        Self::SUB,
        Self::MUL,
        Self::DIV,
        Self::MOD,
        Self::BITAND,
        Self::BITOR,
        Self::BITXOR,
        Self::SHL,
        Self::SHR,
    ];

    pub fn to_string_safe(&self) -> String {
        match self {
            BinaryOp::ADD => String::from("wrapping_add"),
            BinaryOp::SUB => String::from("wrapping_sub"),
            BinaryOp::MUL => String::from("wrapping_mul"),
            BinaryOp::DIV => String::from("checked_div"),
            BinaryOp::MOD => String::from("checked_rem"),
            BinaryOp::SHL => String::from("wrapping_shl"),
            BinaryOp::SHR => String::from("wrapping_shr"),
            BinaryOp::BITAND | BinaryOp::BITOR | BinaryOp::BITXOR => {
                panic!("Bitwise ops are emitted as native operators")
            }
        }
    }
}

impl ToString for BinaryOp {
    fn to_string(&self) -> String {
        match self {
            BinaryOp::ADD => String::from("+"),
            BinaryOp::SUB => String::from("-"),
            BinaryOp::MUL => String::from("*"),
            BinaryOp::DIV => String::from("/"),
            BinaryOp::MOD => String::from("%"),
            BinaryOp::BITAND => String::from("&"),
            BinaryOp::BITOR => String::from("|"),
            BinaryOp::BITXOR => String::from("^"),
            BinaryOp::SHL => String::from("<<"),
            BinaryOp::SHR => String::from(">>"),
        }
    }
}

pub enum CastSource {
    Int(ArithmeticExpr),
    Bool(BoolExpr),
}

pub struct CastExpr {
    source: CastSource,
    target: IntTypeID,
}

impl CastExpr {
    pub fn new_from_int(source: ArithmeticExpr, target: IntTypeID) -> Self {
        CastExpr {
            source: CastSource::Int(source),
            target,
        }
    }

    pub fn new_from_bool(source: BoolExpr, target: IntTypeID) -> Self {
        CastExpr {
            source: CastSource::Bool(source),
            target,
        }
    }

    pub fn as_arith_expr(self) -> ArithmeticExpr {
        ArithmeticExpr::Cast(Box::new(self))
    }

    pub fn get_type(&self) -> TypeID {
        TypeID::IntType(self.target)
    }
}

impl ToString for CastExpr {
    // Both the source expression and the whole cast are parenthesized:
    // `as` has tricky precedence (e.g. `a as u8 + b` binds as
    // `(a as u8) + b`), and a fully parenthesized form sidesteps it.
    fn to_string(&self) -> String {
        match &self.source {
            CastSource::Int(expr) => {
                format!("(({}) as {})", expr.to_string(), self.target.to_string())
            }
            // bool only casts to numeric types directly via an integer step;
            // chaining through u8 keeps it valid for every target width.
            CastSource::Bool(expr) => {
                format!(
                    "(({}) as u8 as {})",
                    expr.to_string(),
                    self.target.to_string()
                )
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum UnaryOp {
    NOT,
    NEG,
}

impl UnaryOp {
    pub const ALL: &'static [Self] = &[Self::NOT, Self::NEG];
}

pub struct UnaryExpr {
    expr: ArithmeticExpr,
    op: UnaryOp,
}

impl UnaryExpr {
    pub fn new(expr: ArithmeticExpr, op: UnaryOp) -> Self {
        UnaryExpr { expr, op }
    }

    pub fn as_arith_expr(self) -> ArithmeticExpr {
        ArithmeticExpr::Unary(Box::new(self))
    }

    pub fn get_type(&self) -> TypeID {
        self.expr.get_type()
    }

    // Negation must be wrapping_neg, never a plain unary minus: `-iN::MIN`
    // panics under overflow-checks=on and wraps when off, so a bare `-`
    // would create false divergences across fuzz configs. wrapping_neg is
    // identical in every config (and is defined for unsigned types too).
    pub fn to_string_safe(&self) -> String {
        match self.op {
            UnaryOp::NOT => format!("(!({}))", self.expr.to_string()),
            UnaryOp::NEG => format!("(({}).wrapping_neg())", self.expr.to_string()),
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

    #[test]
    fn int_cast_has_correct_string_representation() {
        let source = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(30));
        let cast = CastExpr::new_from_int(source, IntTypeID::U8);

        assert_eq!(cast.get_type(), TypeID::IntType(IntTypeID::U8));
        assert_eq!(cast.to_string(), "((30i32) as u8)");
    }

    #[test]
    fn bool_cast_has_correct_string_representation() {
        use crate::program::expr::bool_expr::BoolValue;
        let source = BoolValue::new(true).as_bool_expr();
        let cast = CastExpr::new_from_bool(source, IntTypeID::I64);

        assert_eq!(cast.get_type(), TypeID::IntType(IntTypeID::I64));
        assert_eq!(cast.to_string(), "((true) as u8 as i64)");
    }

    #[test]
    fn unary_not_has_correct_string_representation() {
        let expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_u16(7));
        let unary = UnaryExpr::new(expr, UnaryOp::NOT);

        assert_eq!(unary.get_type(), TypeID::IntType(IntTypeID::U16));
        assert_eq!(unary.to_string_safe(), "(!(7u16))");
    }

    #[test]
    fn unary_neg_uses_wrapping_neg() {
        let expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_i8(5));
        let unary = UnaryExpr::new(expr, UnaryOp::NEG);

        assert_eq!(unary.to_string_safe(), "((5i8).wrapping_neg())");
    }

    #[test]
    fn shifts_use_wrapping_form_with_u32_amount() {
        let left = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(30));
        let right = ArithmeticExpr::new_from_int_expr(IntExpr::new_u64(5));
        let shl = BinaryExpr::new(left, right, BinaryOp::SHL);
        assert_eq!(shl.to_string_safe(), "(30i32).wrapping_shl((5u64) as u32)");

        let left = ArithmeticExpr::new_from_int_expr(IntExpr::new_u8(3));
        let right = ArithmeticExpr::new_from_int_expr(IntExpr::new_i16(2));
        let shr = BinaryExpr::new(left, right, BinaryOp::SHR);
        assert_eq!(shr.to_string_safe(), "(3u8).wrapping_shr((2i16) as u32)");
    }
}
