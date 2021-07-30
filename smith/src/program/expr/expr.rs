use crate::program::var::Var;

use super::arithmetic_expr::{ArithmeticExpr, IntExpr};
use super::bool_expr::BoolExpr;
use super::borrow_expr::BorrowExpr;
use super::for_loop_expr::ForLoopExpr;
use super::struct_expr::StructExpr;

// The top most form of an expression
pub enum Expr {
    Arithmetic(ArithmeticExpr),
    Boolean(BoolExpr),
    Literal(LiteralExpr),
    Variable(Var),
    Borrow(Box<BorrowExpr>),
    Loop(ForLoopExpr),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Self::Literal(s) => s.to_string(),
            Self::Arithmetic(s) => s.to_string(),
            Self::Variable(s) => s.to_string(),
            Self::Boolean(s) => s.to_string(),
            Self::Borrow(s) => (*s).to_string(),
            Self::Loop(s) => s.to_string(),
        }
    }
}

// Literally
pub enum LiteralExpr {
    Int(IntExpr),
    Struct(StructExpr),
    Raw(RawExpr),
}

impl ToString for LiteralExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Int(s) => s.to_string(),
            Self::Struct(s) => s.to_string(),
            Self::Raw(s) => s.to_string(),
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
        Expr::Literal(LiteralExpr::Raw(self))
    }
}

impl ToString for RawExpr {
    fn to_string(&self) -> String {
        self.expr_string.clone()
    }
}
