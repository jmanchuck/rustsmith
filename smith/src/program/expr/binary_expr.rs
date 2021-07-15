use crate::program::types::TypeID;

use super::expr::ArithmeticExpr;

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
}

impl BinaryOp {
    pub fn to_string_safe(&self) -> String {
        match self {
            BinaryOp::ADD => String::from("safe_add"),
            BinaryOp::SUB => String::from("safe_sub"),
            BinaryOp::MUL => String::from("safe_mul"),
            BinaryOp::DIV => String::from("safe_div"),
        }
    }
}

impl ToString for BinaryOp {
    fn to_string(&self) -> String {
        match self {
            Self::ADD => String::from("+"),
            Self::SUB => String::from("-"),
            Self::MUL => String::from("*"),
            _ => String::from("/"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::program::expr::{expr::ArithmeticExpr, int_expr::IntExpr};
    #[test]
    fn binary_expr_has_correct_string_representation() {
        let left_expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(30));
        let right_expr = ArithmeticExpr::new_from_int_expr(IntExpr::new_i32(5));

        // binary expression 30 + 5
        let binary_expr = BinaryExpr::new(left_expr, right_expr, BinaryOp::ADD);

        assert_eq!(binary_expr.to_string(), "30 + 5".to_string());
    }
}
