use crate::program::var::Var;

use super::arithmetic_expr::ArithmeticExpr;
use super::{expr::Expr, func_call_expr::FunctionCallExpr};

pub enum BoolExpr {
    Bool(BoolValue),
    Binary(Box<BinBoolExpr>),
    Comparison(Box<ComparisonExpr>),
    Negation(Box<NegationExpr>),
    Var(Var),
    Func(FunctionCallExpr),
}

impl BoolExpr {
    pub fn as_expr(self) -> Expr {
        Expr::Boolean(self)
    }
}

impl ToString for BoolExpr {
    fn to_string(&self) -> String {
        match self {
            Self::Bool(s) => s.to_string(),
            Self::Binary(s) => (*s).to_string(),
            Self::Comparison(s) => (*s).to_string(),
            Self::Negation(s) => (*s).to_string(),
            Self::Var(s) => s.to_string(),
            Self::Func(s) => s.to_string(),
        }
    }
}

impl From<Expr> for BoolExpr {
    fn from(expr: Expr) -> Self {
        if let Expr::Boolean(result) = expr {
            result
        } else {
            panic!("Could not perform conversion from Expr to BoolExpr")
        }
    }
}

impl From<FunctionCallExpr> for BoolExpr {
    fn from(expr: FunctionCallExpr) -> Self {
        BoolExpr::Func(expr)
    }
}

impl From<Var> for BoolExpr {
    fn from(expr: Var) -> Self {
        BoolExpr::Var(expr)
    }
}

pub struct BoolValue {
    value: bool,
}

impl BoolValue {
    pub fn new(value: bool) -> Self {
        BoolValue { value }
    }

    pub fn as_bool_expr(self) -> BoolExpr {
        BoolExpr::Bool(self)
    }
}

impl ToString for BoolValue {
    fn to_string(&self) -> String {
        if self.value {
            String::from("true")
        } else {
            String::from("false")
        }
    }
}

pub struct BinBoolExpr {
    left: BoolExpr,
    right: BoolExpr,
    op: BoolOp,
}

impl BinBoolExpr {
    pub fn new(left: BoolExpr, right: BoolExpr, op: BoolOp) -> Self {
        BinBoolExpr { left, right, op }
    }

    pub fn as_bool_expr(self) -> BoolExpr {
        BoolExpr::Binary(Box::new(self))
    }
}

impl ToString for BinBoolExpr {
    fn to_string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.to_string(),
            self.op.to_string(),
            self.right.to_string()
        )
    }
}

pub struct ComparisonExpr {
    left: ArithmeticExpr,
    right: ArithmeticExpr,
    op: ComparisonOp,
}

impl ComparisonExpr {
    pub fn new(left: ArithmeticExpr, right: ArithmeticExpr, op: ComparisonOp) -> Self {
        ComparisonExpr { left, right, op }
    }

    pub fn as_bool_expr(self) -> BoolExpr {
        BoolExpr::Comparison(Box::new(self))
    }
}

impl ToString for ComparisonExpr {
    fn to_string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.to_string(),
            self.op.to_string(),
            self.right.to_string()
        )
    }
}

pub struct NegationExpr {
    expr: BoolExpr,
}

impl NegationExpr {
    pub fn new(expr: BoolExpr) -> Self {
        NegationExpr { expr }
    }

    pub fn as_bool_expr(self) -> BoolExpr {
        BoolExpr::Negation(Box::new(self))
    }
}

impl ToString for NegationExpr {
    fn to_string(&self) -> String {
        format!("!({})", self.expr.to_string())
    }
}

pub enum ComparisonOp {
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Equal,
    NotEqual,
}

impl ToString for ComparisonOp {
    fn to_string(&self) -> String {
        match self {
            Self::Greater => String::from(">"),
            Self::Less => String::from("<"),
            Self::GreaterEq => String::from(">="),
            Self::LessEq => String::from("<="),
            Self::Equal => String::from("=="),
            Self::NotEqual => String::from("!="),
        }
    }
}

pub enum BoolOp {
    OR,
    AND,
}

impl ToString for BoolOp {
    fn to_string(&self) -> String {
        match self {
            Self::AND => String::from("&&"),
            Self::OR => String::from("||"),
        }
    }
}
