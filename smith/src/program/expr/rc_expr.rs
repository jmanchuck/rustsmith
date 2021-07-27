#![allow(warnings)]
use crate::program::{types::TypeID, var::Var};

use super::expr::Expr;

pub struct RcExpr {
    expr: Expr,
    type_id: TypeID,
    is_clone: bool,
}

impl RcExpr {
    pub fn new(expr: Expr, type_id: TypeID) -> Self {
        RcExpr {
            expr,
            type_id,
            is_clone: false,
        }
    }
    pub fn new_clone(var: Var, type_id: TypeID) -> Self {
        RcExpr {
            expr: var.as_expr(),
            type_id,
            is_clone: true,
        }
    }
}

impl ToString for RcExpr {
    fn to_string(&self) -> String {
        if self.is_clone {
            format!("Rc::clone({})", self.expr.to_string())
        } else {
            format!("Rc::new({})", self.expr.to_string())
        }
    }
}
