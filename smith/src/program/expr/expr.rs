use crate::program::types::TypeID;
use crate::program::var::Var;

use super::borrow_expr::BorrowExpr;
use super::func_call_expr::FunctionCallExpr;
use super::int_expr::IntExpr;
use super::struct_expr::StructExpr;
use super::{binary_expr::BinaryExpr, bool_expr::BoolExpr};

use strum_macros::{EnumCount, EnumDiscriminants, EnumIter};

pub enum Expr {
    Arithmetic(ArithmeticExpr),
    Boolean(BoolExpr),
    Literal(LiteralExpr),
    Variable(Var),
    Borrow(Box<BorrowExpr>),
}

impl ToString for Expr {
    fn to_string(&self) -> String {
        match self {
            Self::Literal(s) => s.to_string(),
            Self::Arithmetic(s) => s.to_string(),
            Self::Variable(s) => s.to_string(),
            Self::Boolean(s) => s.to_string(),
            Self::Borrow(s) => (*s).to_string(),
        }
    }
}

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
#[derive(EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(ArithmeticExprVariants))]
#[strum_discriminants(derive(EnumCount, EnumIter))]
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
