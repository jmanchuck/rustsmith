use crate::program::var::Var;

use super::arithmetic_expr::ArithmeticExpr;
use super::bool_expr::BoolExpr;
use super::borrow_expr::BorrowExpr;
use super::for_loop_expr::ForLoopExpr;
use super::struct_expr::StructExpr;

// The top most form of an expression
pub enum Expr {
    Arithmetic(ArithmeticExpr),
    Boolean(BoolExpr),
    Struct(StructExpr),
    Variable(Var),
    Borrow(Box<BorrowExpr>),
    Loop(Box<ForLoopExpr>), // ForLoopExpr has large size, use box for reduced enum size
    Raw(RawExpr),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Expr::Arithmetic(s) => s.to_string(),
            Expr::Boolean(s) => s.to_string(),
            Expr::Struct(s) => s.to_string(),
            Expr::Variable(s) => s.to_string(),
            Expr::Borrow(s) => (*s).to_string(),
            Expr::Loop(s) => (*s).to_string(),
            Expr::Raw(s) => s.to_string(),
        }
    }
}

// Using string only to represent the expression
// This should not be used unless needed as it doesn't reflect the program's AST
pub struct RawExpr {
    expr_string: String,
}

impl RawExpr {
    pub fn new(expr_string: String) -> Self {
        RawExpr { expr_string }
    }

    pub fn as_expr(self) -> Expr {
        Expr::Raw(self)
    }
}

impl ToString for RawExpr {
    fn to_string(&self) -> String {
        self.expr_string.clone()
    }
}
